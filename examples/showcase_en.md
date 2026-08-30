---
title: "Doc2Flow English Showcase"
subtitle: "Comprehensive test document for all present and future features"
date: "2026-07-25"
version: "1.0.0"
language: "en"
numbered_sections: true
comments: true
---

:::variables
| Variable | Value |
| --- | --- |
| SERVER_NAME | prod-srv-01 |
| PORT | 8080 |
| API_KEY | secret-key-en-12345 |
:::

# Part 1: System Setup & Preparation

This top-level section describes the basic system configuration with **rich text formatting** such as `inline code`, *italic hints*, and ~~deprecated options~~. Tasks within H1 sections do not display an individual badge indicator, but contribute fully to the overall progress tracking.

- [x] General **safety briefing** for technicians completed (incl. `ISO 27001` & ~~legacy protocol v1~~)

## Section 1: Overview & Guidelines

This is an arbitrary text paragraph in the section body demonstrating a comprehensive set of **text formatting styles**: **Bold text** for emphasis, *italic text* for subtle notes, ***bold and italic combinations*** for critical callouts, `inline code` like `systemctl restart service` for commands/variables, and ~~strikethrough text~~ for outdated specifications (e.g. ~~default port 80~~ replaced by `8080`).

![Example System Architecture](../resources/images/example1.jpg)

![External Remote Image](https://picsum.photos/600/300)

![Broken External Image](https://invalid-host-doc2flow.test/broken-image.jpg)

<!-- Test comment: This comment must not appear in the generated HTML -->

> This is a neutral Note callout box providing standard context along with **bold keywords**, `inline code` paths, and *italic references*.

>? This is a green Tip box offering **best practices**: use `curl -I` instead of ~~telnet~~ for *quick diagnostic checks*.

>! This is a purple Important box highlighting **critical requirements** (`TLS v1.3` mandatory, ~~SSL v3~~ disabled).

>!! This is a yellow Warning box advising **caution** for insecure ports (`{{PORT}}`) and *unencrypted transmissions*.

>!!! This is a red Caution box warning against **dangerous operations** such as `rm -rf /` or ~~unauthenticated writes~~.

> This is a multi-line callout box containing detailed **background information** and comprehensive instructions for the end user. It spans across multiple sentences to demonstrate fluid text wrapping and clean alignment with `inline code elements`, *italic annotations*, **highlighted key terms**, and ~~strikethrough legacy items~~ within visual callout containers.

## Section 2: Task Checklist

### Setup & Preparation
This arbitrary text paragraph describes the preliminary setup steps with **priority tiers**, `file paths`, and ~~obsolete requirements~~ that must be fulfilled before marking individual items as completed.

- [ ] **Unpack hardware** and verify components against serial `SN-2026-X`
  - [ ] Inspect accessory completeness (*manual*, `power cable`, ~~adapter v1~~)
  - [x] Check casing for transport damage (incl. **visual inspection** of ports)
- [ ] **Connect network and power supply** (ensure `VLAN 10` assignment)
  1. Plug primary network cable into **Port 1** (`eth0`, `10 Gbps`)
  2. Plug redundant network cable into **Port 2** (`eth1`, ~~1 Gbps fallback~~ `10 Gbps`)
- [x] Initial power-on check completed (**status LED** shows `GREEN`)

[External System Documentation](https://example.com/docs)

![Download System Specification PDF](https://example.com/files/specification.pdf)

#### Software Configuration
- [ ] Install latest system updates via `apt update && apt upgrade -y` (verify **kernel** `6.x`)
- [ ] Configure firewall rules according to company policy (open port `{{PORT}}`, close ~~port 21 FTP~~)
- [x] Verify remote access service (`sshd` active on port `22`, *password auth* disabled)

![Software Configuration Overview](../resources/images/example2.jpg)

# Part 2: Maintenance & System Reference

## Section 3: Reference & Code Blocks

### System Registry Settings
The following configuration snippet must be applied to disable automatic updates (set `DisableWindowsUpdateAccess=1`, remove ~~legacy key~~):

```ini
[HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate]
"DisableWindowsUpdateAccess"=dword:00000001
"Server"="{{SERVER_NAME}}"
```

### Unlabelled Script Block
```
echo "Initializing Doc2Flow showcase environment {{SERVER_NAME}}:{{PORT}} with key {{API_KEY}}..."
```

### System Components & Specifications

The table below outlines the hardware specifications of the installed system components with **model details**, `interfaces`, and status indicators:

| Component | Model | Status | Capacity |
|---|---|---|---|
| **Central Processor** | `Intel Xeon E-2388G` | **Active** | 8 Cores / 16 Threads |
| **System Memory** | `DDR4 ECC Registered` | *Optimal* | **64 GB** (2x 32 GB, ~~32 GB minimum~~) |
| **Primary Storage** | `NVMe SSD PCIe 4.0` | *Normal* | **2 TB** RAID 1 |
| **Network Interface** | `Dual 10GbE SFP+` | **Connected** | `10 Gbps` (~~1 Gbps legacy~~) |

## Section 4: Informational Lists

- **Main task & system monitoring** (standard operation `system-daemon`)
  1. Sub-item A 1: Query service status with `systemctl is-active` (**status:** *active*)
  2. Sub-item A 2: Analyze error log `/var/log/syslog` (ignore ~~legacy logfile~~)
- **Procedure for maintenance tasks** (security classification `Level 2`)
  - [ ] Prepare test run with parameters `--dry-run` and `--verbose`
     - Detailed parameter check `X` (*threshold* `> 95%`)
     - Detailed parameter check `Y` (~~standard timeout 30s~~ now `60s`)
  - [x] Approval by **system administrator** (authorized via `signature token`)

1. Sequential main step 1: **Initialization** using `init --force`
   - Subordinate verification step 1.1: Validate `config.json` (*schema v2*, ~~v1 deprecated~~)
   - Subordinate verification step 1.2: Verify certificate chain (`cert.pem`)
2. Sequential main step 2: **Data transfer** execution
   1. Detailed sub-procedure 2.a: Establish connection to `{{SERVER_NAME}}`
   2. Detailed sub-procedure 2.b: Perform data synchronization via port `{{PORT}}` (*encrypted*)
      1. Protocol handshake validation (`TLS 1.3` session key exchange)
      2. Chunked stream encoding with integrity verification
         1. Block checksum verification via SHA-256 buffer check
         2. Throughput monitoring and adaptive rate limiting
            1. Bandwidth saturation analysis (target threshold `> 100 MB/s`)
            2. Packet loss telemetry logging (max tolerance `< 0.01%`)
3. Sequential main step 3: **Finalization & verification** (archive audit log `audit.log`)

## Unknown Elements

:::unknown-directive
This is an unknown directive.
- List 1
- List 2
:::

::unnamed-directive[Label]{Attribute}
Another unknown directive with label and attribute.
Here is also an unknown inline :directive[unknown] in this line.

---

![System Information Overview](../resources/images/example3.jpg)

![White Background Test Image](../resources/images/example4.png)

# Empty Top-Level Section

## Empty Sub-Section
