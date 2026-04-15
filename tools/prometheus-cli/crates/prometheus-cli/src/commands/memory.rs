use anyhow::Result;
use colored::Colorize;
use prometheus_learn::memory::SurrealMemoryClient;

pub async fn ping() -> Result<()> {
    println!("{}", "🧠 Checking surreal-memory...".bold());

    let client = SurrealMemoryClient::from_env()
        .ok_or_else(|| anyhow::anyhow!("SURREAL_MEMORY_URL not set"))?;

    match client.ping().await {
        Ok(true) => println!("  {} Server healthy at {}", "✅".green(), client.base_url()),
        Ok(false) => println!("  {} Server responded but unhealthy", "⚠️".yellow()),
        Err(e) => println!("  {} Cannot reach server: {}", "❌".red(), e),
    }

    Ok(())
}

pub async fn stats() -> Result<()> {
    println!("{}", "🧠 Surreal-Memory Stats".bold());

    let client = SurrealMemoryClient::from_env()
        .ok_or_else(|| anyhow::anyhow!("SURREAL_MEMORY_URL not set"))?;

    let entities = client.search("*", None).await?;
    println!("  Entities: {}", entities.len().to_string().cyan());

    Ok(())
}

pub async fn search(query: &str, entity_type: Option<&str>) -> Result<()> {
    println!("{} {}", "🔍 Searching:".bold(), query);

    let client = SurrealMemoryClient::from_env()
        .ok_or_else(|| anyhow::anyhow!("SURREAL_MEMORY_URL not set"))?;

    let results = client.search(query, entity_type).await?;
    println!("  Found {} result(s):\n", results.len());

    for entity in &results {
        println!("  {} [{}] {}", "▸".cyan(), entity.entity_type.dimmed(), entity.name.bold());
        for obs in entity.observations.iter().take(3) {
            println!("    {}", obs.dimmed());
        }
    }

    Ok(())
}
