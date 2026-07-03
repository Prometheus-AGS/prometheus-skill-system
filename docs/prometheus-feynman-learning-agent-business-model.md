# Prometheus Feynman Learning Agent — Business Model & Monetization Strategy

## Document Control

| Field | Value |
|---|---|
| **Project** | Prometheus Feynman Learning Agent (PFLA) |
| **Version** | 1.0.0-draft |
| **Date** | 2026-07-01 |
| **Status** | Draft for Review |
| **Related Documents** | `prometheus-feynman-learning-agent-architecture.md`, `prometheus-feynman-learning-agent-functional-spec.md`, `prometheus-feynman-learning-agent-implementation-plan.md` |

---

## 1. Executive Summary

The Prometheus Feynman Learning Agent (PFLA) is positioned as a **premium AI-native learning platform** that helps anyone master any subject using the Feynman Technique. The business model follows a **freemium-to-subscription** path with three tiers: **Free**, **Plus**, and **Pro**, and extends into a **three-sided marketplace** where students, certified masters, and the platform form a learning economy. The marketplace features include the **Coach Catalog** (digital coaching personas), **Creator Studio** (expert persona creation), **Master Certification** (student-to-master pipeline), **Live Coaching** (text, async video, and WebRTC video), and a **Revenue Share Engine** that handles all marketplace transactions. The full marketplace business model is detailed in the **Business Model Addendum** (`prometheus-feynman-learning-agent-business-model-addendum.md`).

**Key Metrics (Target at 12 Months)**:
- Monthly Active Users (MAU): 50,000
- Paid Subscriber Conversion: 8-12% (industry average for productivity tools: 2-5%; education tools: 5-10%)
- Average Revenue Per User (ARPU): $12/month
- Monthly Recurring Revenue (MRR): $48,000-$72,000
- Monthly Active Masters: 100 (marketplace beta)
- Monthly Coaching Sessions: 2,000 (marketplace beta)
- Gross Merchandise Value (GMV): $100,000/month (marketplace beta)

**Revenue Mix (Target at 24 Months)**:
- Subscription revenue: 50%
- Enterprise/Team licenses: 15%
- Affiliate/API usage: 5%
- Marketplace live coaching revenue: 25%
- Creator Studio revenue: 5%

---

## 2. Value Proposition & Target Market

### 2.1 Core Value Proposition

> **"Learn anything deeply. Never forget. The AI tutor that uses the Feynman Technique to find your gaps and fix them."**

Unlike generic AI chatbots (ChatGPT, Claude) that provide answers but don't ensure understanding, PFLA:
1. **Forces active recall**: You must explain the concept yourself.
2. **Finds hidden gaps**: AI grading identifies what you don't know you don't know.
3. **Fixes recursively**: Drills down into gaps until mastery is real, not assumed.
4. **Remembers forever**: Spaced retention checks prevent forgetting.
5. **Works offline**: Local-first data means your learning journey is always available.

### 2.2 Target Personas

| Persona | Demographics | Pain Points | Willingness to Pay |
|---|---|---|---|
| **The Self-Directed Learner** | 22-35, university or early career. Learns programming, data science, languages, or complex topics online. | Overwhelmed by resources; can't tell if they truly understand; forgets after "learning." | High — $10-20/mo for a tool that ensures mastery. |
| **The Career Switcher** | 30-45, transitioning to tech or a new field. Needs to learn efficiently. | Needs to learn fast but deeply; can't afford gaps in knowledge; interviews require real understanding. | Very High — $20-50/mo for accelerated, verified learning. |
| **The Lifelong Student** | 40-60, intellectually curious. Learns physics, history, philosophy for personal growth. | Wants to understand, not just consume; values retention and connection between concepts. | Medium — $10-15/mo for a premium, ad-free experience. |
| **The Educator / Trainer** | 25-55, teacher, corporate trainer, or content creator. | Needs to ensure students truly understand; wants data on student gaps; wants to scale personalized feedback. | High (B2B) — $50-200/mo per seat for team licenses. |
| **The Parent** | 35-50, wants to help children learn effectively. | Wants to ensure children understand homework, not just memorize; wants to track progress. | Medium — $15-20/mo for family plan. |

### 2.3 Total Addressable Market (TAM)

| Segment | Market Size (2026) | Growth Rate | Notes |
|---|---|---|---|
| **Online Learning (B2C)** | $350B global | 15% CAGR | Includes MOOCs, tutoring, language learning apps. |
| **AI Tutoring / EdTech** | $12B global | 45% CAGR | Fastest-growing subsegment; post-ChatGPT boom. |
| **Spaced Repetition / Flashcards** | $800M global | 20% CAGR | Anki, Quizlet, Memrise; PFLA is a premium alternative. |
| **Corporate Training (B2B)** | $370B global | 10% CAGR | L&D budgets shifting to AI-powered personalized learning. |

**Serviceable Addressable Market (SAM)**: $2B (AI-native tutoring + spaced repetition + premium self-directed learning in North America, Europe, and East Asia).

**Serviceable Obtainable Market (SOM)**: $20M (0.1% of SAM in Year 1; realistic for a niche premium product with strong word-of-mouth).

---

## 3. Freemium Architecture

### 3.1 Free Tier Design Philosophy

The Free tier is **generous enough to create habit** but **limited enough to create upgrade pressure**. The Feynman loop itself is the core hook — the Free tier gives users the full novice-level experience so they feel the value, but restrictions on depth, retention, and cross-platform access create natural upgrade triggers.

**Free Tier Features**:
- 3 active learning goals at a time.
- Full Feynman loop at **novice** audience level only.
- Basic analogies and explanations.
- Immediate grading and gap identification.
- 1 level of recursion (child loops on gaps).
- Web-only access (no desktop app, no mobile app).
- Community support only.
- No retention scheduling ("Did you forget?" is the most powerful upgrade trigger).

**Free Tier Limitations** (designed to trigger upgrade):
- **Retention**: After mastering a concept, the Free user sees: *"Retention check scheduled. Upgrade to Plus to never forget."* This is the **primary upgrade trigger** — the pain of forgetting is visceral.
- **Depth**: Free users hit a ceiling at novice level. When they try to explain to a "peer" or "skeptic," they see: *"Upgrade to Plus for deeper mastery levels."*
- **Goals**: The 3-goal limit is reached quickly by active learners (≈ 2-3 weeks). The UI shows: *"2/3 goals used. Upgrade for unlimited."*
- **Offline**: Free users can use the web app, but there's no offline guarantee (no local-first push). The prompt: *"Upgrade to Plus for offline learning on any device."*

### 3.2 Plus Tier ($9.99/month or $99/year — 17% discount)

**Plus Tier Value Proposition**: *"The serious learner. Master any subject with unlimited depth and never forget."*

**Plus Features**:
- Unlimited active goals.
- All audience levels: **novice**, **peer**, **skeptic**.
- **Retention scheduling**: 24h, 7d, 30d, 90d spaced repetition checks.
- **Artifact library**: Full history of all explanations, grades, and mastery badges. Exportable to Markdown/PDF.
- **Desktop app**: Tauri desktop app for Windows, macOS, Linux.
- **Offline mode**: Full local-first with PGlite + ElectricSQL sync.
- **Priority support**: Email support with 24h response time.
- **Basic MCP tools**: Web search, calculator, knowledge query (from the user's own artifact library).

**Plus Upgrade Triggers** (in-app moments of need):
1. **Retention Reminder**: When a Free user masters a concept, show a banner: *"Your brain will start forgetting this in 24 hours. Upgrade to Plus to schedule a retention check."*
2. **Peer Level Lock**: When a Free user completes novice mastery, show: *"You're ready to explain this to a peer. Upgrade to Plus to test deeper understanding."*
3. **Goal Limit**: When creating the 4th goal, show a modal: *"You've reached 3 goals. Archive an existing goal or upgrade to Plus for unlimited."*
4. **Offline Need**: When the user loses network, show: *"You're offline. Upgrade to Plus for full offline learning with automatic sync."*

### 3.3 Pro Tier ($19.99/month or $199/year — 17% discount)

**Pro Tier Value Proposition**: *"The power learner. Optimize your learning with AI-driven insights, custom tools, and priority access."*

**Pro Features** (everything in Plus, plus):
- **Karpathy Loop Insights**: Personal learning analytics dashboard showing:
  - Your Learning Velocity Score (LVS) over time.
  - Concepts you learn fastest vs. slowest.
  - Recommended learning strategies based on your data.
  - Comparison to anonymized cohort averages.
- **Custom MCP Tools**: Configure your own MCP servers (e.g., personal knowledge base, company wiki, custom code execution environment).
- **Priority LLM Queue**: Faster response times during peak hours (guaranteed < 2s first token).
- **Mobile app**: iOS and Android apps (when available).
- **Advanced A2UI Surfaces**: Interactive diagrams, code playgrounds, 3D visualizations (when supported by the catalog).
- **API Access**: Personal API key for integrating PFLA into your own tools (e.g., Notion plugin, Obsidian plugin).
- **Priority Support**: Live chat support with 4h response time.
- **Early Access**: Beta features (new audiences, new tool types, new visualizations).

**Pro Upgrade Triggers**:
1. **Analytics Dashboard Teaser**: Show a "Preview" of the LVS chart with blurred data: *"Upgrade to Pro to see your full learning analytics."*
2. **Custom Tools Teaser**: When a user asks for a tool that isn't in the default set, show: *"Upgrade to Pro to connect your own tools and data sources."*
3. **Speed Teaser**: During peak hours, show a message: *"Pro users get priority access. Upgrade for faster responses."*

### 3.4 Enterprise / Team Tier (Custom Pricing, $50-$200/seat/month)

**Target**: Corporate L&D departments, universities, training organizations.

**Team Features**:
- Everything in Pro.
- **Team Goals**: Shared learning goals for teams (e.g., "All engineers must master Rust by Q3").
- **Manager Dashboard**: View team progress, concept mastery heatmaps, gap analysis across the team.
- **Custom Curricula**: Upload organization-specific curricula and knowledge bases.
- **SSO / SAML**: Enterprise authentication.
- **On-premise Option**: Deploy PFLA on organization's own infrastructure (Flint-Forge backend + Tauri desktop + custom MCP servers).
- **Dedicated Support**: Slack/Teams channel with dedicated success manager.
- **SLA**: 99.9% uptime guarantee, < 1h incident response.

**Pricing**: Custom based on seat count, curriculum complexity, and support level. Minimum 25 seats.

---

## 4. Unit Economics & Financial Model

### 4.1 Cost Structure (Per Active User / Month)

| Cost Category | Free | Plus | Pro | Notes |
|---|---|---|---|---|
| **LLM API Costs** | $0.50 | $3.00 | $5.00 | GPT-4o average: ~$0.01/1K tokens. Feynman loop ≈ 50K tokens per concept. 3 loops/month for Free, 20 for Plus, 40 for Pro. |
| **Infrastructure** | $0.10 | $0.20 | $0.30 | Postgres, ElectricSQL sync, CDN, Axum server hosting. |
| **Storage** | $0.05 | $0.10 | $0.20 | PGlite artifacts, sync history, media. |
| **Payment Processing** | $0 | $0.30 | $0.60 | Stripe: 2.9% + $0.30 per transaction. |
| **Support** | $0 | $0.50 | $1.50 | Email vs. live chat; Pro requires more human support. |
| **Total COGS** | **$0.65** | **$4.10** | **$7.60** | |
| **Gross Margin** | **N/A** | **59%** | **62%** | Target: 60%+ gross margin. |

### 4.2 Lifetime Value (LTV) Analysis

| Tier | Monthly Price | Avg. Churn/Month | Avg. Lifetime | LTV | LTV/CAC |
|---|---|---|---|---|---|
| **Plus** | $9.99 | 8% | 12.5 months | $124.88 | 3.1x |
| **Pro** | $19.99 | 5% | 20 months | $399.80 | 4.0x |
| **Enterprise** | $100 | 3% | 33 months | $3,300 | 5.0x |

*Assumptions*: CAC for Plus = $40 (paid ads + content marketing), CAC for Pro = $100 (plus upgrades from Plus), CAC for Enterprise = $660 (sales-led).

**Target**: LTV/CAC ≥ 3x for sustainability; ≥ 5x for aggressive growth.

### 4.3 Revenue Projection (24 Months)

| Month | Free Users | Plus Users | Pro Users | Enterprise Seats | MRR | ARR |
|---|---|---|---|---|---|---|
| 1 | 1,000 | 50 | 10 | 0 | $699 | $8,388 |
| 3 | 5,000 | 300 | 50 | 0 | $3,997 | $47,964 |
| 6 | 15,000 | 1,000 | 200 | 50 | $14,990 | $179,880 |
| 9 | 30,000 | 2,500 | 500 | 150 | $39,975 | $479,700 |
| 12 | 50,000 | 4,000 | 800 | 300 | $69,960 | $839,520 |
| 18 | 80,000 | 6,500 | 1,500 | 600 | $124,935 | $1,499,220 |
| 24 | 120,000 | 10,000 | 2,500 | 1,000 | $199,900 | $2,398,800 |

*Assumptions*: 8% Free→Plus conversion, 20% Plus→Pro conversion, 0.5% Free→Enterprise conversion (via sales). Churn: 8% Plus, 5% Pro, 3% Enterprise.

### 4.4 Break-Even Analysis

| Cost Category | Monthly (at 12-month scale) | Notes |
|---|---|---|
| **Engineering Team** | $40,000 | 4 engineers @ $120K/year average. |
| **Design / Product** | $8,000 | 1 designer + 1 PM @ $96K/year average. |
| **Marketing** | $15,000 | Paid ads, content, influencer partnerships. |
| **Infrastructure** | $5,000 | AWS/Cloudflare + Postgres + LLM API credits. |
| **Legal / Finance / Admin** | $3,000 | LLC, accounting, compliance, insurance. |
| **Total Burn** | **$71,000/month** | |
| **Break-Even MRR** | **$71,000** | Achieved at ~Month 14-16 with the above user projections. |
| **Runway** | 18 months | With $500K seed funding. |

---

## 5. Go-to-Market Strategy

### 5.1 Launch Phases

| Phase | Timing | Channel | Goal | Tactics |
|---|---|---|---|---|
| **Alpha** | Month 0-2 | Private invite (500 users) | Validate loop, find bugs, get testimonials. | Founder network, Twitter/X DMs, Reddit private invites. |
| **Beta** | Month 3-5 | Public waitlist (5,000 users) | Build buzz, iterate on pricing, test retention. | Product Hunt "Upcoming" page, Hacker News teaser, newsletter. |
| **Public Launch** | Month 6 | Full public | Maximize acquisition, prove unit economics. | Product Hunt launch, Hacker News "Show HN", influencer partnerships, paid ads. |
| **Growth** | Month 7-12 | Organic + paid | Scale to 50K MAU, 5K paid. | Content marketing (Feynman Technique guides, "How I learned X in Y days" blog posts), YouTube tutorials, SEO. |
| **Enterprise** | Month 12+ | Sales-led | Land first 10 enterprise customers. | LinkedIn outreach, conference booths (EdTech, L&D), case studies. |

### 5.2 Acquisition Channels

| Channel | CAC | Target Volume | Strategy |
|---|---|---|---|
| **Organic / SEO** | $0 | 30% of signups | Blog posts on Feynman Technique, learning science, AI in education. Rank for "Feynman technique app," "AI tutor," "learn anything fast." |
| **Product Hunt / Hacker News** | $0 | 10% of signups | Launch strategically (Tuesday 12:01 AM PT for PH). Craft compelling maker story. |
| **Twitter/X / LinkedIn** | $0 | 15% of signups | Founder and team post daily learning tips, product updates, user success stories. Build in public. |
| **Paid Social (Meta, TikTok)** | $25 | 20% of signups | Video ads showing the Feynman loop in action (30-60 seconds). Target: 22-35, interests in self-improvement, coding, online courses. |
| **Paid Search (Google)** | $40 | 15% of signups | Target keywords: "best way to learn," "how to remember what you learn," "AI tutor," "spaced repetition app." |
| **Referral Program** | $10 | 10% of signups | "Give 1 month free, get 1 month free" for Plus/Pro referrals. |
| **Partnerships** | $0 | Variable | Integrate with Notion, Obsidian, Anki. Co-marketing with course creators (e.g., "Master Andrew Ng's ML course with PFLA"). |

### 5.3 Retention Strategy

Retention is the **key driver of LTV** in subscription businesses. PFLA's retention is built into the product design:

1. **Habit Formation**: The Feynman loop itself is a daily/weekly habit. Push notifications (mobile) and system tray reminders (desktop) for retention checks create recurring engagement.
2. **Progress Visibility**: The mastery tree, badges, and LVS score are "sticky" — users don't want to lose their progress.
3. **Streaks**: "You have a 14-day learning streak. Don't break it!" (gamification, but meaningful).
4. **Social Proof**: Optional sharing of mastery badges to Twitter/X, LinkedIn. "I mastered Quantum Entanglement with PFLA."
5. **Community**: Discord server with channels per subject. Peer learning increases retention.
6. **Curriculum Depth**: The recursion and escalation system means there's always a deeper level to reach. A user who masters "novice" will want to try "peer."

**Retention Metrics to Track**:
- D1/D7/D30 retention (free and paid separately).
- Loop completion rate (% of started loops that reach mastery).
- Days since last loop (churn risk indicator: > 7 days = at risk).
- Retention check pass rate (higher = better product-market fit).

---

## 6. Competitive Positioning

### 6.1 Competitive Landscape

| Competitor | Price | Strengths | Weaknesses | Our Differentiation |
|---|---|---|---|---|
| **ChatGPT / Claude** | $20/mo | General-purpose, fast, cheap. | No structured learning, no gap identification, no retention, no offline. | Structured Feynman loop with grading and recursion. |
| **Khan Academy** | Free | High-quality content, structured courses. | Passive consumption, no AI tutor, no personalization, no gap finding. | Active learning with AI-driven gap detection and fixing. |
| **Quizlet / Anki** | Free-$35/yr | Spaced repetition, flashcards. | Rote memorization, no deep understanding, no explanation generation. | Conceptual mastery with transfer problems, not just memorization. |
| **Duolingo** | Free-$7/mo | Gamified, habit-forming, mobile-first. | Surface-level learning, no deep concepts, no gap analysis. | Deep, recursive learning for any subject, not just languages. |
| **Coursera / Udemy** | $30-$100/course | Structured courses, certificates. | Passive video watching, no active recall, no personalized gap fixing. | Active learning loop that works with any course or resource. |
| **TutorMe / Wyzant** | $30-$80/hr | Human tutor, personalized. | Expensive, scheduling friction, not scalable, no retention tracking. | AI tutor available 24/7, cheaper, with structured retention. |
| **BibiGPT** | $9.99/mo | Video summarization, mind maps, Feynman templates. | Video-centric, no recursive loop, no offline, no native apps. | Universal subject support, recursive loop, offline, desktop/mobile. |

### 6.2 Positioning Statement

> **For self-directed learners who want to truly master complex subjects, PFLA is the AI-native learning platform that uses the Feynman Technique to find your knowledge gaps and fix them recursively. Unlike generic AI chatbots or passive video courses, PFLA forces active explanation, grades your understanding, drills into gaps, and ensures you never forget — all while working offline.**

---

## 7. Pricing Experiments & A/B Testing

### 7.1 Experiment Roadmap

| Experiment | Hypothesis | Metric | Duration | Success Criteria |
|---|---|---|---|---|
| **Price Anchor Test** | Showing annual pricing first increases annual subscriptions. | Annual plan % | 4 weeks | Annual share increases from 20% to 35%. |
| **Retention Upsell Timing** | Upselling retention at 24h (vs. at mastery) increases conversion. | Plus conversion from Free | 4 weeks | Conversion increases by 20%. |
| **Pro Feature Teaser** | Showing blurred LVS chart to Plus users increases Pro upgrades. | Plus→Pro conversion | 4 weeks | Conversion increases by 15%. |
| **Family Plan** | A family plan (2-5 users) at $15/mo increases LTV. | Family plan uptake | 6 weeks | > 10% of Plus users switch to family. |
| **Enterprise Trial** | 14-day enterprise trial increases sales pipeline. | Enterprise trial→close rate | 8 weeks | Trial close rate > 20%. |
| **Pay-What-You-Want** | A "pay what you want" option for Plus (min $5) increases conversion in developing markets. | Conversion in emerging markets | 6 weeks | Conversion increases by 30% in target markets. |

### 7.2 Pricing Psychology

1. **Decoy Effect**: Pro tier at $19.99 makes Plus at $9.99 look like a bargain.
2. **Anchoring**: Annual price ($99) shown first, then monthly ($9.99) as "or $9.99/month."
3. **Loss Aversion**: "Your retention check is due. Don't lose your mastery. Upgrade now."
4. **Social Proof**: "10,000+ learners have mastered 50,000+ concepts with PFLA."
5. **Endowment Effect**: Free users who create 3 goals feel ownership; losing them (by archiving) triggers upgrade.

---

## 8. Risk & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| **Low conversion from Free to Paid** | Medium | High | Optimize upgrade triggers; A/B test pricing; offer longer free trials; reduce Free tier limits gradually. |
| **High LLM costs eroding margins** | High | High | Implement aggressive caching; use cheaper models for simple tasks; offer local LLM option for Pro users; negotiate volume discounts. |
| **Competitor builds similar feature** | Medium | Medium | Deepen moat: proprietary grading rubrics, Karpathy Loop data advantage, Flint-Forge integration complexity, community. |
| **Churn due to "learning fatigue"** | Medium | High | Implement "maintenance mode" (minimal retention checks only); celebrate milestones; allow pausing subscriptions. |
| **Enterprise sales cycle too long** | Medium | Medium | Focus on product-led growth first; offer self-serve team plans (5-25 seats) before sales-led. |
| **Regulatory risk (AI in education)** | Low | Medium | Comply with GDPR, COPPA (no under-13), FERPA (for enterprise). Avoid storing PII in LLM prompts. |
| **Economic downturn reduces discretionary spending** | Medium | High | Emphasize ROI ("Learn a new skill, get a better job"); offer hardship discounts; strengthen free tier. |

---

## 9. Success Metrics & KPIs

### 9.1 North Star Metric

**Learning Velocity Score (LVS) per user per month** — the average number of concepts mastered per learner per month. This directly measures the product's core value: helping people learn faster and deeper.

### 9.2 KPI Dashboard

| Category | Metric | Target (Month 6) | Target (Month 12) | Target (Month 24) |
|---|---|---|---|---|
| **Acquisition** | MAU | 10,000 | 50,000 | 120,000 |
| | New signups/month | 3,000 | 10,000 | 20,000 |
| | CAC | $35 | $30 | $25 |
| **Activation** | % completing first loop | 40% | 50% | 60% |
| | Time to first loop | < 5 min | < 3 min | < 2 min |
| **Retention** | D30 retention (Free) | 20% | 25% | 30% |
| | D30 retention (Paid) | 60% | 65% | 70% |
| | Monthly churn (Paid) | 10% | 8% | 5% |
| **Revenue** | MRR | $5,000 | $70,000 | $200,000 |
| | ARR | $60,000 | $840,000 | $2,400,000 |
| | ARPU | $8 | $12 | $15 |
| | LTV/CAC | 2.5x | 3.5x | 4.5x |
| **Engagement** | Loops started / user / month | 5 | 8 | 10 |
| | Mastery rate (% loops reaching mastery) | 30% | 40% | 50% |
| | Retention check pass rate | 70% | 75% | 80% |
| **Product** | NPS | 30 | 40 | 50 |
| | App Store rating | 4.2 | 4.5 | 4.7 |
| | Support tickets / 1000 users | 50 | 30 | 20 |

---

## 10. Appendix: Subscription Enforcement Technical Design

### 10.1 Backend Enforcement

```rust
// crates/pfla-api/src/middleware/subscription.rs
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

pub async fn require_subscription(
    tier: SubscriptionTier,
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let user = req.extensions().get::<AuthenticatedUser>()
        .ok_or(ApiError::Unauthorized)?;
    
    if user.subscription_tier < tier {
        return Err(ApiError::SubscriptionRequired {
            required: tier,
            current: user.subscription_tier,
        });
    }
    
    Ok(next.run(req).await)
}

// Router configuration
let app = Router::new()
    .route("/api/v1/goals", post(create_goal))
    .layer(middleware::from_fn_with_state(
        state.clone(),
        |state, req, next| require_subscription(SubscriptionTier::Free, state, req, next),
    ))
    .route("/api/v1/learn/feynman-loop", post(start_loop))
    .layer(middleware::from_fn_with_state(
        state.clone(),
        |state, req, next| require_subscription(SubscriptionTier::Free, state, req, next),
    ))
    .route("/api/v1/learn/feynman-loop", post(start_loop_with_audience))
    .layer(middleware::from_fn_with_state(
        state.clone(),
        |state, req, next| require_subscription(SubscriptionTier::Plus, state, req, next),
    ));
```

### 10.2 Frontend Enforcement

```typescript
// ui/src/lib/subscription.ts
export enum SubscriptionTier {
  Free = 'free',
  Plus = 'plus',
  Pro = 'pro',
}

export const TIER_FEATURES: Record<SubscriptionTier, string[]> = {
  [SubscriptionTier.Free]: ['goals:3', 'audience:novice', 'platform:web'],
  [SubscriptionTier.Plus]: ['goals:unlimited', 'audience:all', 'retention', 'platform:desktop', 'offline'],
  [SubscriptionTier.Pro]: ['goals:unlimited', 'audience:all', 'retention', 'platform:all', 'offline', 'analytics', 'custom_mcp', 'api_access'],
};

export function hasFeature(tier: SubscriptionTier, feature: string): boolean {
  return TIER_FEATURES[tier].includes(feature);
}

// ui/src/components/FeatureGate.tsx
export function FeatureGate({
  feature,
  children,
  fallback,
}: {
  feature: string;
  children: React.ReactNode;
  fallback?: React.ReactNode;
}) {
  const { tier } = useSubscription();
  
  if (hasFeature(tier, feature)) {
    return <>{children}</>;
  }
  
  return fallback || (
    <UpgradePrompt
      feature={feature}
      requiredTier={getMinimumTier(feature)}
    />
  );
}
```

### 10.3 Stripe Webhook Handler

```rust
// crates/pfla-api/src/handlers/billing.rs
pub async fn stripe_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let payload = stripe::Webhook::construct_event(
        &body,
        headers.get("Stripe-Signature").and_then(|h| h.to_str().ok()),
        &state.stripe_webhook_secret,
    )?;
    
    match payload.event_type {
        EventType::InvoicePaid => {
            let invoice: Invoice = payload.data.deserialize()?;
            update_subscription_tier(&state.db, &invoice.customer, SubscriptionTier::Plus).await?;
        }
        EventType::InvoicePaymentFailed => {
            let invoice: Invoice = payload.data.deserialize()?;
            schedule_downgrade(&state.db, &invoice.customer, SubscriptionTier::Free).await?;
        }
        EventType::CustomerSubscriptionUpdated => {
            let sub: Subscription = payload.data.deserialize()?;
            let tier = map_stripe_price_to_tier(&sub.items.data[0].price);
            update_subscription_tier(&state.db, &sub.customer, tier).await?;
        }
        _ => {}
    }
    
    Ok(StatusCode::OK)
}
```

---

*End of Business Model & Monetization Strategy*
