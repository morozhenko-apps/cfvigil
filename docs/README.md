# CFVigil documentation

Keep product behavior, source checks, native acceptance and publication evidence
separate. These are reusable contracts, not per-run journals.

| Topic | Owner |
| --- | --- |
| Product behavior and boundaries | [Product](PRODUCT.md) |
| Development and permanent branches | [Agent guide](../AGENTS.md) |
| Verification and release gates | [Delivery](DELIVERY.md) |
| Source and security acceptance | [QA matrix](qa/MODE_B_MATRIX.md) |
| Signed flat publication and recovery | [Distribution](DISTRIBUTION.md) |
| Linux/Windows installation and credentials | [Native acceptance](NATIVE_MACHINE_GATE.md) |
| Connected provider behavior | [Cloudflare smoke](NATIVE_CLOUDFLARE_SMOKE.md) |
| Windows-specific acceptance | [Windows smoke](WINDOWS_NATIVE_SMOKE.md) |
| Icon regeneration and identity | [Icon pipeline](ICON_PIPELINE.md) |
| Localization contracts | [Architecture](I18N_ARCHITECTURE.md), [context review](i18n/CONTEXT_AUDIT.md) |
| Product direction and issue ownership | [Direction](ROADMAP_TO_1_0.md) |

Commit-specific results belong in Actions/issues. Private credential custody,
provider identifiers, local locations and operational exports stay outside this
public repository. Retain every unique product, security and native acceptance
requirement; removing documentation never erases Git history.
