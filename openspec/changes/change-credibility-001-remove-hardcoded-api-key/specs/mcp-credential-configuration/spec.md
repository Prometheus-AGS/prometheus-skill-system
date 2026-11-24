## ADDED Requirements

### Requirement: Require a configured Tavily credential

The MCP configurator SHALL obtain the Tavily credential from TAVILY_API_KEY, report its absence explicitly, and leave embedded credentials out of distributed scripts. A secret scan SHALL detect committed credential literals, and the operator’s rotation of the exposed credential SHALL be recorded before closing the historical repair.

#### Scenario: Configure an installation without a credential

- **WHEN** the operator configures the Tavily MCP endpoint with TAVILY_API_KEY unset
- **THEN** the configurator reports the missing setting and uses no embedded credential
