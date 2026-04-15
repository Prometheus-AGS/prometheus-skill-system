// ================================================================
// Prometheus Skill Mutation Policies
// ================================================================
// Default-deny: Cedar denies all requests unless a permit matches.
// These policies govern the self-learning pipeline's write operations
// against skill artifacts.
// ================================================================

// --- DEVELOPMENT: permit all operations ---

permit(
    principal,
    action,
    resource
) when {
    context.environment == "development"
};

// --- STAGING: require validation for mutations ---

permit(
    principal,
    action == Action::"skill.mutate",
    resource
) when {
    context.environment == "staging" &&
    context has validation_passed &&
    context.validation_passed == true
};

// --- STAGING: require human approval for promotions ---

permit(
    principal,
    action == Action::"skill.promote",
    resource
) when {
    context.environment == "staging" &&
    context has human_approved &&
    context.human_approved == true &&
    context has test_pass_rate &&
    context.test_pass_rate >= 95
};

// --- STAGING: allow generation with validation ---

permit(
    principal in AgentGroup::"generators",
    action == Action::"skill.generate",
    resource
) when {
    context.environment == "staging" &&
    context has validation_passed &&
    context.validation_passed == true
};

// --- TRACE CAPTURE: always allow with governance consent ---

permit(
    principal,
    action == Action::"trace.capture",
    resource
) when {
    context has governance_consent &&
    context.governance_consent == true
};

// --- PRODUCTION: deny all mutations by default ---

forbid(
    principal,
    action == Action::"skill.mutate",
    resource
) when {
    context.environment == "production"
};

forbid(
    principal,
    action == Action::"skill.generate",
    resource
) when {
    context.environment == "production"
};

// --- HEALTHCARE VERTICAL: require audit trail ---

forbid(
    principal,
    action == Action::"skill.mutate",
    resource
) when {
    context has vertical &&
    context.vertical == "healthcare" &&
    !(context has audit_trail_id)
};

// --- FINANCIAL VERTICAL: require dual approval ---

forbid(
    principal,
    action == Action::"skill.promote",
    resource
) when {
    context has vertical &&
    context.vertical == "financial" &&
    !(context has dual_approval && context.dual_approval == true)
};
