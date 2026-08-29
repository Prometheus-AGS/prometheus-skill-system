# Goals — openspec-mirror-drift-cleanup› sovereign-sync-service-reliability

- Determine why sovereign-sync repeatedly exits and is not restarted
- Implement a durable service recovery and restart fix
- Verify sovereign-sync remains healthy and no longer blocks KBD
- Return control to the parent phase and resume its exact next command
