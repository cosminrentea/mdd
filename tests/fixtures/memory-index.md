# Agent Memory Index

## Pre-Task Rules
- [Read CLAUDE.local.md Before Any Repo Work](feedback-read-claude-local.md) — Must read repo's CLAUDE.local.md before touching any other file in it

## User
- [User Profile](user-profile.md) — Cosmin Rentea, Sr. Engineer, Commerce Catalog Storefront, Europe/Bucharest

## Feedback
- [Jenkins ITC HOME Override](jenkins-itc-migration.md) — HOME="${env.WORKSPACE}" breaks git identity; remove it
- [CiliumNetworkPolicy Pathfinder DNS](cilium-eks-pathfinder-dns.md) — toFQDNs on EKS needs Pathfinder DNS egress (169.254.20.10)
- [Agent Guidance & Workflows](feedback-agent-guidance.md) — Pitfalls, ambiguity questions, testing rules, PR workflow patterns, data-source-first investigation, validation completeness
- [Git Hooks: No AI Agent Refs](feedback-git-hooks.md) — Global commit-msg hook rejects Claude/Codex/Co-Authored-By in commits
- [obs/ Write Rules](feedback-obs-structure.md) -- Coding agents write to obs/lrn/; obs/memory/ written by lrn-import, auto-memory, dream
- [playwright-cli Interactive Sessions](feedback-playwright-cli.md) -- Must use --headed for user auth; full checklist

## Reference — Infrastructure & Ops
- [AWS Cost & Billing API Access](reference-aws-cost-access.md) — KLAM profiles to CUR 2.0 mapping, resource-level CE 14-day limit, account policy availability
- [Jenkins Operations](ref-jenkins-ops.md) — API auth, CMR jobs (PaaS/Cell/Close), job naming, release/deploy flows, ATS K8s migration, HTTPS PAT expiry, e2e-on-prem branch switching, load-testing-ingestion KLAM/SQS
- [Flex/ArgoCD Operations](ref-flex-argocd.md) — ArgoCD auth (Azure AD, 90-day refresh), sync timing, ValidateDeployment chartPath, Helm subchart standalone validation and `required` guards
- [Deploy Infrastructure](ref-infrastructure.md) — BBC factory (golang v120-v124 only), per-cell config, VaultSecret CRD, Vault broker paths, s6-overlay CRI-O workaround, broker flex-deploy subchart pattern
- [Splunk Access & Investigation](ref-splunk.md) — PaaS/Flex Splunk indexes, sourcetype patterns, log structure, cross-service trace correlation methodology, playwright-cli approach
- [Cell/SaaS Splunk Instances](ref-splunk-cell.md) — Separate instances per region (splunk-us/eu/ap), indexes ccssms_nonprod/prod
- [GitHub Auth & CLI](ref-github-auth.md) — GHE Banyan workarounds, github.com multi-account access, draft PR conversion, merge strategies per repo
- [K8s Access Limitations](k8s-access-limitations.md) — Azure AD RBAC: QA accessible, Stage/Prod blocked; Splunk log workaround for restricted namespaces
- [K8s PaaS vs Flex Naming](ref-k8s-naming.md) — Namespace/pod naming discriminators (PaaS vs Flex), known QA namespaces, aggregator hybrid gotcha, corp endpoints (no port-forward)
- [VPN Protocol Blocks & Toggle How-To](ref-vpn-protocol-blocks.md) — VPN blocks SFTP/gRPC; PES sync E2E toggle protocol (file-based signaling); daemon-automated VPN toggle; cloudflared http2 workaround
- [Cross-Cluster Networking](ref-cross-cluster-networking.md) — ethos501→ethos340 cross-VPC unreachable; use public API Gateway
- [Slack PR Announcements](slack-pr-channel.md) — Post to #apollo-tech-internal (C0311DQG00P)
- [Cortex Monitoring Access](ref-cortex-access.md) — nacho/pacho endpoints, 6 tenants (PaaS/DS-Flex/Cell x nonprod/prod), promtool CLI config, label case conventions, credential/Vault paths

## Reference — Services & Domain
- [Magento Domain Knowledge](ref-magento-domain.md) — Headers, customer groups, pricing semantics, visibleIn pipeline, CCDM vs PaaS models, local ports, common-lib config keys
- [ProductBus (Helix Commerce)](ref-productbus.md) — PB repo inventory, CF Worker architecture, rendering paths, indexing pipeline, mixerConfig/productIndexerConfig requirements
- [Scoping Service Reference](ref-scoping-service.md) — Purpose, MongoDB data model (scopes/channels/policies), ACO ownership
- [Tiers-Calculator & Fair Usage](ref-tiers-calculator.md) — Go architecture, SaaS deployment additions, flex-deploy env name mapping, ConfigMap watch, PaaS rollout validation checklist
- [catalog-service-sync & Callers](ref-catalog-service-sync.md) — Read-only gRPC service; dual PaaS+Cell deploy, data-management callers, CIF/bulk-pdp-ssg impact, Splunk profiling data
- [Events Service Patterns](ref-events-service.md) — Multi-module extraction (P1), V1 operation propagation (P2), delete events via cron (2026-04-03), EventPublisherAspect limitations, shared CCDM/V1 path
- [Cleanup Tenant Flow](ref-cleanup-flow.md) — Two-stage tenant cleanup (SQS soft-delete + SNS fan-out), E2E Docker cleanup, EventPublisherAspect gaps, saas:resync cannot send deletion markers
- [MongoDB Atlas & Query Patterns](ref-mongodb-patterns.md) — mongosh IAM assume-role access, sandbox cluster details (7 collections), query shape identification (service attribution)
- [Kafka Consumer Patterns](ref-kafka-patterns.md) — Events-service Kafka Streams, ParallelConsumer pattern, Adobe Pipeline requirements, CLI access limitation (IMS Java-only), Cell QA topic, Guava bug
- [ASR & Spring Boot Gotchas](ref-asr-spring.md) — ASR connector-core missing transitive deps, management port test issues (@LocalManagementPort), protobuf 3.x/4.x version pin for Google SDKs
- [Stateful Job Patterns](ref-stateful-job-patterns.md) — ShedLock only in cloud-manager, no checkpoint/resume in fleet
- [CCM Add-on Model](ref-ccm-addons.md) — UPP fulfillment, Floodgate flags, ConfigurableFiCode — three layers of per-org feature/limit control
- [CPS Architecture Scope](ref-cps-architecture.md) — Pure downstream executor, no entitlement awareness; all commercial logic in CCM

## Reference — PES (Product Export Service)
- [PES Technical Details](pes-details.md) — Module structure (5 modules, protobuf version isolation), GMC WebClient ops, enrichment, E2E validation procedure, GMC2 status, shared test utilities, Service Account auth, SFTP port/naming, rebase patterns
- [PES Architecture & Design](ref-pes-architecture.md) — Control/data plane separation, dual data paths, GMC DataSource design (separate primaries per merchant), file delivery (SFTP vs CloudFront), write semantics, blockers, risk registry
- [Flink/Poppie Composite Model](ref-flink-poppie.md) — Composite product model, content hash diff per bit, SNS delivery modes (UPDATE/DELETE/FULL_EXPORT), poppie-sdk full export, Flink-to-GMC surgical updateMask patching, Cell QA S3 findings
- [Poppie gRPC Endpoints](ref-poppie-endpoints.md) — All Cell (7 envs) + PaaS (3 envs) gRPC endpoints, hostname patterns, ethos cluster mapping, AWS accounts, corp endpoint derivation
- [GMC Validation & Issue APIs](ref-gmc-validation-apis.md) — fileUploads, ProductStatus, DS attribution semantics, aggregateProductStatuses, renderissues; feed quotas

## Reference — Shell & Local Dev
- [zsh sourced scripts pitfalls](reference-zsh-sourced-scripts.md) — zsh has no trap RETURN; set -e in sourced files kills parent shell

## Reference — Knowledge Management
- [distill→Skill Graduation](ref-distill-graduation.md) — Pattern for promoting memory → obs/distill/ → skill; rejection criteria

## Reference — Tooling
- [GitHub Copilot PR Reviewer](ref-github-copilot-reviewer.md) — Bot type requires UI or REST API with capital-C "Copilot", not gh CLI
- [Commerce MCP Tools](ref-commerce-mcp-tools.md) — Tool name mismatch (snake/camel), default Adobe Store tenant, chat client header limitation
- [agent-browser CLI Gotchas](ref-agent-browser.md) — Extension needs --headed + fresh daemon; eval injection alternative
- [Shopping Buddy Extension](ref-shopping-buddy.md) — Source repos, adobestore.com-only activation, hardcoded backend URL
- [claude-mem Setup & Architecture](ref-claude-mem.md) — Azure OpenAI provider (NOT Gemini), DB/source patches for v12.4.1, architecture/storage layout, worker recovery, post-upgrade checklist
- [FluffyJaws MCP Tools](ref-fluffyjaws.md) — Recommended fj tools (full_documentation_search, fluffyjaws_investigation, slack_search); overlap/exclusions with existing MCPs
- [Serena MCP Setup & Gotchas](ref-serena-mcp.md) — JDT LS credential fix, silent failure modes, path quirks, evaluation (8.4/10)
- [Scout Installation & Gotchas](ref-scout-setup.md) — v0.9.54 setup (39 repos), CLAUDE.md creation hazard, symlink resolution, daemon details (~3GB RSS), workspace skill path
- [Scout vs SocratiCode Benchmark](ref-scout-vs-socraticode.md) — 9-test benchmark: Scout wins symbol/architecture (7W), SocratiCode wins cross-repo (1W); agent prioritization gap
- [mq & qmd Markdown Query Patterns](ref-mq-patterns.md) -- mq selectors (.yaml for frontmatter), qmd line-offset retrieval, grep-based heading filters
- [QMD Embedding Model Upgrade](reference-qmd-upgrade.md) -- Gemma-300M to Qwen3-0.6B (384 to 1024 dims), timeout/context bug patches, config and patch file locations

## Reference — Load Testing
- [STG NA1 Ingestion Load Testing](reference-load-testing-stg-na1.md) -- Cell STG NA1 load test findings (aggregator bottleneck, broker DLQ cascades), namespace-to-service mapping, Locust GIL constraints, DATA-7135

## Project
- [Infrastructure Status](project-infra-status.md) — PaaS ethos501/502→ethos340/341 migration (verified 2026-04-01), Cortex RBAC gaps on ethos340

## Domain Knowledge
- [Domain Knowledge](domain-knowledge.md) — Aggregator/CCDM field semantics, ACO CatalogView structure, ChannelModel rename

# Unimported learnings
- Recent observations = in `obs/lrn/` files after `_import.md` date
- Periodically imported via `/lrn-import` skill
