# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 2.x.x   | :all:              |

## Reporting a Vulnerability

Please do not report a vulnerability via Github issues. Any vulnerability shall be reported by emailing to Pana.Yang@hotmail.com or via our official website security reporting section (under construction).

## Developer Security Standards (Mandatory for Organization Members and Recommended for Contributors)

### **I. Authorized Operating Environment**

* **Recommended OS:** Fedora KDE Plasma Desktop
* **Disk Encryption:** LUKS (Linux Unified Key Setup) **must** be enabled by default during installation.

### **II. Mandatory Security Suite**

To ensure system integrity and real-time threat detection, the following tools must be installed and configured:

* **Endpoint Monitoring:** Wazuh-agent
* **Hardening & Auditing:** SELinux (set to `Enforcing`), firewalld, Lynis
* **Threat Scanning:** ClamAV, rkhunter, chkrootkit
* **System Integrity:** AIDE (Advanced Intrusion Detection Environment), unhide
* **Network Scanning:** Daily Nmap scan of the local network (e.g., `192.168.1.0/24`)
* **Security Server:** Wazuh-server (includes Wazuh-dashboard, Wazuh-manager and Wazuh-indexer), Greenbone Community Edition

### **III. Identity & Access Management**

* **2FA Requirement:** Hardware-based authentication via **YubiKey** is mandatory for all organizational accounts and SSH access.

---

### Appendix

```text
# sysctl settings are defined through files in
# /usr/lib/sysctl.d/, /run/sysctl.d/, and /etc/sysctl.d/.
#
# Vendors settings live in /usr/lib/sysctl.d/.
# To override a whole file, create a new file with the same in
# /etc/sysctl.d/ and put new settings there. To override
# only specific settings, add a file with a lexically later
# name in /etc/sysctl.d/ and put new settings there.
#
# For more information, see sysctl.conf(5) and sysctl.d(5).

# --- A. Network Hardening ---

# Protect against IP spoofing (source validation)
net.ipv4.conf.all.rp_filter = 1
net.ipv4.conf.default.rp_filter = 1

# Ignore ICMP broadcast requests to avoid being part of Smurf attacks
net.ipv4.icmp_echo_ignore_broadcasts = 1

# Ignore bad ICMP errors (prevents some types of attacks)
net.ipv4.icmp_ignore_bogus_error_responses = 1

# Disable IP source routing (usually unnecessary and exploitable)
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.default.accept_source_route = 0

# Disable secure ICMP redirects
net.ipv4.conf.all.secure_redirects = 0
net.ipv4.conf.default.secure_redirects = 0

# Log spoofed packets, source routed packets, and redirects
net.ipv4.conf.all.log_martians = 1

# Protection against SYN flood attacks (enables SYN cookies)
net.ipv4.tcp_syncookies = 1

# Increase the maximum number of connections waiting for acceptance
net.core.somaxconn = 4096
net.core.netdev_max_backlog = 5000

# --- B. Filesystem / Permission Hardening ---

# Disable magic SysRq key (unless you need it for emergency debugging)
kernel.sysrq = 0

# Enable restriction of link/symlink traversal (to prevent user links in world-writable dirs)
fs.protected_hardlinks = 1
fs.protected_symlinks = 1

# Restrict the permissions of dmesg to non-root users (hiding kernel messages from attackers)
kernel.dmesg_restrict = 1

# Restrict unprivileged users from using the bpf() system call
kernel.unprivileged_bpf_disabled = 1

# Randomize the virtual address space layout (ASLR) - Default is usually 2, ensure it's on
kernel.randomize_va_space = 2

# --- C. Resource and Memory Hardening ---

# Prevent non-root users from writing to /proc/sys/vm/*
kernel.yama.protected_sysctl = 1 

# Restrict the dumping of memory (core dumps)
fs.suid_dumpable = 0

# Controls memory allocation behavior - often set to 1 or 2. 
# A value of 2 ensures no overcommit, helping prevent stability issues when memory runs out.
vm.overcommit_memory = 1
vm.max_map_count=262144

# Restrict non-root user create too many User Namespace
user.max_user_namespaces = 10000

# ---D. Yama Settings ---

# Restrict ptrace to only ancestor processes (prevents snooping)
kernel.yama.ptrace_scope = 1
```

```text
0 2 * * * /usr/bin/freshclam --quiet
5 2 * * * /usr/bin/flock -n /tmp/clamscan.lock /usr/bin/nice -n 19 /usr/bin/ionice -c 3 /usr/bin/clamdscan --multiscan --fdpass --quiet / >> /var/log/clamav-scan-$(date +\%Y\%m\%d).log
10 2 * * * /usr/sbin/chkrootkit >> /var/log/chkrootkit-scan-$(date +\%Y\%m\%d).log
15 2 * * * /usr/bin/aide --check
0 3 1 * * /usr/bin/logger "REMINDER: Run 'aide --update' after a system upgrade to maintain database integrity."
20 2 * * * /usr/sbin/unhide proc tcp udp sys >> /var/log/unhide-scan-$(date +\%Y\%m\%d).log 2>&1
40 2 * * * /usr/bin/lynis --cronjob --quiet >> /var/log/lynis-report-$(date +\%Y\%m\%d).log
50 2 * * * /usr/bin/rkhunter --cronjob --rwo >> /var/log/rkhunter-scan-$(date +\%Y\%m\%d).log
0 4 * * * /usr/local/bin/daily_nmap_scan.sh >/dev/null 2>&1
```

```shell
#!/bin/bash

# Define variables
TIMESTAMP=$(date +"%Y-%m-%d")
SCAN_DIR="/var/log/nmap_scans"
TARGET_SPEC="192.168.1.0/24"

# Create the log directory if it doesn't exist
mkdir -p $SCAN_DIR

# Run the Nmap scan
/usr/bin/nmap -A -T4 $TARGET_SPEC -oN $SCAN_DIR/nmap_scan_$TIMESTAMP.txt
```
