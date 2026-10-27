# Connected Cloudflare verification

Use a limited-permission read-only test account and compare observations with Cloudflare's own interface. Keep credentials, account identifiers and private project names out of public issues and screenshots.

Check that Workers and Pages are discovered independently, that the latest build or deployment attempt controls status, and that opening a resource goes to the correct Cloudflare page. Verify running, failed, successful and unknown states without generating fake production failures.

Repeat after losing network access, receiving a provider error and restarting the app. A failed query must not erase a separate valid response or turn an unknown state into green. The history and notification stream should reflect genuine transitions rather than the mere act of connecting.

For Worker error spikes and zone-level WAF events, verify data availability and account scope separately. Missing analytics never proves an incident cleared. Zone security signals do not establish which Worker is affected.

Store detailed results with the associated test or bug report, using redacted evidence. This file is a reusable test checklist, not a chronological account log.
