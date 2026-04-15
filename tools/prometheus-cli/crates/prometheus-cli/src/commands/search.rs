use anyhow::Result;
use colored::Colorize;

pub async fn run(query: &str, limit: usize) -> Result<()> {
    println!("{} {}", "🔍 Searching GitHub for:".bold(), query);

    let octocrab = octocrab::instance();
    let results = octocrab
        .search()
        .repositories(&format!("{} topic:agent-skills", query))
        .per_page(limit as u8)
        .send()
        .await?;

    if results.items.is_empty() {
        println!("  No results found.");
        return Ok(());
    }

    println!("  Found {} result(s):\n", results.items.len());

    for repo in &results.items {
        let name = &repo.full_name.as_deref().unwrap_or("unknown");
        let desc = repo.description.as_deref().unwrap_or("No description");
        let stars = repo.stargazers_count.unwrap_or(0);
        let has_skills = name.contains("skill");
        let tag = if has_skills { "[skills]".green() } else { "[source]".yellow() };

        println!("  {} {} {} {}", tag, name.bold(), format!("⭐{}", stars).dimmed(), desc);
    }

    Ok(())
}
