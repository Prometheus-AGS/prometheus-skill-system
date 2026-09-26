package __APP_ID__

import com.google.ai.edge.litertlm.Backend
import com.google.ai.edge.litertlm.Content
import com.google.ai.edge.litertlm.Contents
import com.google.ai.edge.litertlm.Conversation
import com.google.ai.edge.litertlm.ConversationConfig
import com.google.ai.edge.litertlm.Engine
import com.google.ai.edge.litertlm.EngineConfig
import com.google.ai.edge.litertlm.Message
import com.google.ai.edge.litertlm.OpenApiTool
import com.google.ai.edge.litertlm.SamplerConfig
import com.google.ai.edge.litertlm.ToolProvider
import com.google.ai.edge.litertlm.tool
import android.util.Log
import android.app.Application
import android.content.pm.ApplicationInfo
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.util.Locale

/**
 * Thin Android runtime binding for Rust-owned local inference.
 *
 * Flutter does not call this class. Rust captures the process JavaVM in
 * JNI_OnLoad and invokes this adapter from gen_ui_inference. Keep policy,
 * prompt/context assembly, tool permissioning, persistence, and event
 * normalization in Rust; this file only binds the official LiteRT-LM Android
 * API.
 */
object __APP_CLASS__LiteRtLmBridge {
    private const val TAG = "__APP_CLASS__LiteRtLmBridge"
    private const val FORCE_TOOL_RESULT_FALLBACK_FILE = "__APP_NAME__-force-litertlm-tool-result-fallback"

    private var engine: Engine? = null
    private var conversation: Conversation? = null
    private var modelPath: String? = null
    private var cacheDir: String? = null

    @JvmStatic
    @Synchronized
    fun load(modelPath: String, cacheDir: String): String {
        if (engine != null && this.modelPath == modelPath && this.cacheDir == cacheDir) {
            return diagnosticsJson("ready")
        }
        close()
        val config = EngineConfig(
            modelPath = modelPath,
            backend = Backend.CPU(),
            visionBackend = null,
            audioBackend = null,
            maxNumTokens = null,
            maxNumImages = null,
            cacheDir = cacheDir,
        )
        val loadedEngine = Engine(config)
        loadedEngine.initialize()
        engine = loadedEngine
        this.modelPath = modelPath
        this.cacheDir = cacheDir
        return diagnosticsJson("loaded")
    }

    @JvmStatic
    @Synchronized
    fun sendMessage(requestJson: String): String {
        val currentEngine = engine ?: error("LiteRT-LM engine is not loaded")
        val request = JSONObject(requestJson)
        val activeConversation = currentEngine.createConversation(
            conversationConfigFromRequest(request),
        )
        conversation = activeConversation

        val messages = request.optJSONArray("messages") ?: JSONArray()
        val lastUser = lastUserContent(messages)
            ?: request.optString("prompt").takeIf { it.isNotBlank() }
            ?: error("request contains no user prompt")
        val response = activeConversation.sendMessage(lastUser)
        return messageToJson(response)
    }

    @JvmStatic
    @Synchronized
    fun sendToolResults(resultsJson: String): String {
        val activeConversation = conversation ?: error("LiteRT-LM conversation is not active")
        val results = JSONArray(resultsJson)
        val responses = mutableListOf<Content.ToolResponse>()
        for (index in 0 until results.length()) {
            val item = results.optJSONObject(index) ?: continue
            val name = item.optString("toolName")
            if (name.isBlank()) continue
            val output = item.optString("outputJson", "{}")
            responses += Content.ToolResponse(name, output)
        }
        if (responses.isEmpty()) {
            error("tool result continuation requires at least one named tool result")
        }
        val response = activeConversation.sendMessage(Message.tool(Contents.of(responses)))
        val forceFallback = shouldForceToolResultFallback()
        if (messageHasVisibleOutput(response) && !forceFallback) {
            return messageToJson(response)
        }
        if (forceFallback) {
            Log.w(TAG, "Forcing LiteRT-LM tool-result fallback for Android certification")
        }

        val followUp = activeConversation.sendMessage(
            "Use the tool result that was just provided to answer the user's original request. " +
                "If the tool result reports an error, explain the error briefly. " +
                "Do not call another tool unless it is required to answer."
        )
        return messageToJson(followUp, toolResultFallbackApplied = true)
    }

    private fun shouldForceToolResultFallback(): Boolean {
        if (!isDebuggableApp()) return false
        val dir = cacheDir ?: return false
        return File(dir, FORCE_TOOL_RESULT_FALLBACK_FILE).exists()
    }

    private fun isDebuggableApp(): Boolean {
        return try {
            val activityThread = Class.forName("android.app.ActivityThread")
            val application = activityThread
                .getMethod("currentApplication")
                .invoke(null) as? Application ?: return false
            (application.applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE) != 0
        } catch (_: Throwable) {
            false
        }
    }

    @JvmStatic
    @Synchronized
    fun cancel() {
        conversation?.cancelProcess()
    }

    @JvmStatic
    @Synchronized
    fun close() {
        conversation?.close()
        conversation = null
        engine?.close()
        engine = null
        modelPath = null
        cacheDir = null
    }

    @JvmStatic
    @Synchronized
    fun diagnosticsJson(): String = diagnosticsJson(if (engine?.isInitialized() == true) "ready" else "not_loaded")

    private fun conversationConfigFromRequest(request: JSONObject): ConversationConfig {
        val messages = request.optJSONArray("messages") ?: JSONArray()
        val initialMessages = mutableListOf<Message>()
        var systemInstruction: Contents? = null
        for (index in 0 until messages.length()) {
            val item = messages.optJSONObject(index) ?: continue
            val role = item.optString("role")
            val content = item.optString("content")
            // Only user turns are safe to drop when empty. Dropping a blank
            // assistant or tool turn breaks call/result pairing: the model would
            // see a tool result with no preceding call (or vice versa) and lose
            // the thread. Rust renders tool calls into `content`, so a blank
            // assistant turn here means genuinely empty output, which we still
            // preserve as a placeholder to keep the transcript ordered.
            if (content.isBlank() && role != "assistant" && role != "tool") continue
            when (role) {
                "system" -> systemInstruction = Contents.of(content)
                "assistant" -> initialMessages += Message.model(Contents.of(nonEmpty(content)))
                "tool" -> initialMessages += Message.tool(Contents.of(nonEmpty(content)))
                // The latest user message is sent through sendMessage() so the
                // Conversation API computes only the incremental prompt.
                "user" -> if (index < messages.length() - 1) initialMessages += Message.user(content)
            }
        }

        val settings = request.optJSONObject("generation_settings")
        val samplerConfig = SamplerConfig(
            settings?.optInt("top_k", 20) ?: 20,
            settings?.optDouble("top_p", 0.8) ?: 0.8,
            settings?.optDouble("temperature", 0.7) ?: 0.7,
            0xC0FFEE.toInt(),
        )

        return ConversationConfig(
            systemInstruction = systemInstruction,
            initialMessages = initialMessages,
            tools = toolProvidersFromRequest(request),
            samplerConfig = samplerConfig,
            automaticToolCalling = false,
            channels = emptyList(),
            extraContext = mapOf("enable_thinking" to false),
        )
    }

    /**
     * Keep a preserved-but-empty turn representable. `Contents.of("")` is not a
     * safe assumption across LiteRT-LM versions, and an ordered transcript with
     * an explicit placeholder beats a silently dropped turn.
     */
    private fun nonEmpty(content: String): String =
        content.ifBlank { "(no output)" }

    private fun toolProvidersFromRequest(request: JSONObject): List<ToolProvider> {
        val tools = request.optJSONArray("tools") ?: return emptyList()
        val providers = mutableListOf<ToolProvider>()
        for (index in 0 until tools.length()) {
            val item = tools.optJSONObject(index) ?: continue
            val name = item.optString("name")
            if (name.isBlank()) continue
            val description = item.optString("description")
            val parameters = item.optJSONObject("parameters") ?: JSONObject().put("type", "object")
            val schema = JSONObject()
                .put("name", name)
                .put("description", description)
                .put("parameters", parameters)
            providers += tool(RegisteredOpenApiTool(schema.toString()))
        }
        return providers
    }

    private fun lastUserContent(messages: JSONArray): String? {
        for (index in messages.length() - 1 downTo 0) {
            val item = messages.optJSONObject(index) ?: continue
            if (item.optString("role").lowercase(Locale.US) == "user") {
                return item.optString("content").takeIf { it.isNotBlank() }
            }
        }
        return null
    }

    private fun messageToJson(message: Message, toolResultFallbackApplied: Boolean = false): String {
        val result = JSONObject()
        result.put("role", message.role.value)
        result.put("text", message.contents.contents.joinToString(separator = "") { contentToText(it) })
        val calls = JSONArray()
        for (toolCall in message.toolCalls) {
            val item = JSONObject()
            item.put("name", toolCall.name)
            item.put("arguments", JSONObject(toolCall.arguments))
            calls.put(item)
        }
        result.put("toolCalls", calls)
        result.put("toolResultFallbackApplied", toolResultFallbackApplied)
        result.put("raw", message.toString())
        return result.toString()
    }

    private fun messageHasVisibleOutput(message: Message): Boolean =
        message.contents.contents.any { contentToText(it).isNotBlank() } ||
            message.toolCalls.isNotEmpty()

    private fun contentToText(content: Content): String =
        when (content) {
            is Content.Text -> content.text
            else -> content.toString()
        }

    private fun diagnosticsJson(state: String): String =
        JSONObject()
            .put("state", state)
            .put("backend", "litert-lm-cpu")
            .put("modelPath", modelPath)
            .put("cacheDir", cacheDir)
            .toString()

    private class RegisteredOpenApiTool(
        private val schemaJson: String,
    ) : OpenApiTool {
        override fun getToolDescriptionJsonString(): String = schemaJson

        override fun execute(paramsJsonString: String): String =
            """{"error":"tool execution is host-managed by Rust"}"""
    }
}
