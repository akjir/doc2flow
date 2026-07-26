---
title: "Onboarding Guide"
subtitle: "Preparation Notebook & M365-User | Customer: NyriSys"
company: "NyriSys Inc"
contact: ""
agent: ""
date: ""
---

## 1 | Goods Receipt & Visual Inspection

> New devices are ordered by the customer or Mr. Mueller directly to ATS in Falkenhain.

- [ ] Unpack notebook
- [ ] Visual inspection: check housing, screen, ports for damage
- [ ] Set up device in ATS lab
- [ ] Connect to power and network
- [ ] Test basic functions: image & sound output
- [ ] Test basic functions: charging function
- [ ] Test basic functions: keyboard & touchpad
- [ ] Test camera & microphone (only possible after Windows login)

### Document hardware in ATS documentation
- [ ] Device name
- [ ] Serial number
- [ ] Product number
- [ ] Device location (if known)
- [ ] Take and insert a product photo of the device

## 2 | Windows Installation

- [ ] Follow Windows installation wizard

### Assign device name
> Schema: Last name (without umlauts) + year of purchase | Example: SCHNEIDER2026

- [ ] Enter device name according to schema

### Create local user
- [ ] "How would you like to set up the device?" -> For work or school -> Next
- [ ] Sign-in options -> "Join a domain instead"
- [ ] Username: ATSadmin
- [ ] Password: leave empty (will be assigned later)

### Adjust power plan
- [ ] Power plan -> High performance / Ultimate performance

## 3 | Create admin users & document in Bitwarden

> Each device receives 2 local admin users: ATSadmin and DeviceAdmin.

### ATSadmin
- [ ] Create ATSadmin user
- [ ] Set password
- [ ] **Create Bitwarden entry**
  Entry name: `NyriSys - Clients - Laptop - "Device Name" - ATSadmin`
  Description: Local admin on notebook "Device Name" for ATS

### DeviceAdmin
- [ ] Create DeviceAdmin user
- [ ] Set password
- [ ] **Create Bitwarden entry**
  Entry name: `NyriSys - Clients - Laptop - "Device Name" - DeviceAdmin`
  Description: Local admin on notebook "Device Name" for ATS

## 4 | Remote Access (TeamViewer)

- [ ] Install TeamViewer
- [ ] Set personal password
- [ ] Note TeamViewer ID
- [ ] **Create Bitwarden entry**
  Entry name: `NyriSys - Clients - Laptop - "Device Name" - TeamViewer - "ID"`
  Description: TeamViewer access for notebook "Device Name"

## 5 | Software Uninstallation

> Remove all unnecessary pre-installed applications.

- [ ] Identify pre-installed bloatware
- [ ] Completely uninstall unnecessary applications
- [ ] Check program overview according to guidelines (see screenshot):

![Fig. 1 - Target state program overview after uninstallation](./images/image1.png)

## 6 | Software Installation

### Create installation folder
- [ ] Create folder: C:\INSTALL or C:\ATS\INSTALL
- [ ] Place all installers in the folder

> DO NOT delete installers after installation - NyriSys guideline!

### Install the following applications
- [ ] 7-Zip
- [ ] Adobe Acrobat Reader (Free)
- [ ] Google Chrome
- [ ] Greenshot
- [ ] Notepad++
- [ ] Paint.NET
- [ ] PDF24
- [ ] Trellix Endpoint Security
- [ ] Canon Printer Driver (Neustadt)

## 7 | Entra Integration & BitLocker

- [ ] Join device to Entra with: ATSadmin@nyrisys.onmicrosoft.com
- [ ] Restart device
- [ ] Log in with ATSadmin@nyrisys.onmicrosoft.com
- [ ] Set Windows Hello PIN
- [ ] **Create Bitwarden entry**
  Entry name: `NyriSys - Clients - Laptop - "Device Name" - Windows Hello PIN - ATSadmin@nyrisys`
  Description: Windows Hello PIN login for ATSadmin@nyrisys.onmicrosoft.com on notebook
- [ ] Print BitLocker key to PDF -> save externally (e.g., at ATS)
- [ ] Log out ATS user

## 8 | Create M365 User

> NyriSys will provide: First name, Last name, Position, Email, Mobile number.

### Required Information
- [ ] First name
- [ ] Last name
- [ ] Position
- [ ] Email address
- [ ] Mobile number (SIM card comes with device)

### Generate password
> NyriSys guideline: Generate a memorable password.

- [ ] Open password generator: [codepalm.de/tools/passwort-generator](https://www.codepalm.de/tools/passwort-generator/)
- [ ] Generate password and store securely in Bitwarden
- [ ] **Create Bitwarden entry**
  Entry name: `NyriSys - M365 - User - "firstname.lastname@nyrisys.de"`
  Description: M365 login for employee FIRSTNAME Lastname

> Configure settings as shown in the screenshot.

![Fig. 2 - Password generator settings (Codepalm)](./images/image2.png)

### License assignment
> Assign standard license (unless otherwise specified).

- [ ] License assigned in M365 portal

### Teams Groups - add all new users to:
- [ ] NyriSys
- [ ] RISK MANAGEMENT
- [ ] PROCESS MANAGEMENT
- [ ] HR PORTAL
- [ ] CUSTOMERS
- [ ] Strategy and Business Models
- [ ] IT Management

> Only add to additional groups after consulting with NyriSys.

## 9 | End-User Setup (with M365 Login)

- [ ] Run Windows setup wizard
- [ ] Skip biometric setup (user's responsibility)
- [ ] Set Windows Hello PIN
- [ ] **Create Bitwarden entry (User)**
  Entry name: `NyriSys - Clients - Laptop - "Device Name" - Windows Hello PIN - firstname.lastname@nyrisys.de`
  Description: Windows Hello PIN login for firstname.lastname@nyrisys.de on notebook "Device Name"

> If the smartphone is not yet available: create a TOTP entry for the user in Bitwarden!

### Set Greenshot to German
- [ ] Change language in Greenshot to German (see screenshot):

![Fig. 3 - Greenshot language setting](./images/image3.png)

### Display scaling
- [ ] Start -> Settings -> System -> Display -> Scaling = 125%

![Fig. 4 - Scaling 125%](./images/image4.png)

### Uninstall applications (new Windows user apps)
- [ ] Provider notification
- [ ] Audio recorder
- [ ] Family
- [ ] Journal
- [ ] Solitaire & Casual Games
- [ ] Start Experiences app
- [ ] Xbox

### Registry Entries: Disable Outlook New
> Import REG file to prevent automatic migration to Outlook New.

- [ ] Import REG value

```ini
; Disable automatic migration to Microsoft Outlook new
[HKEY_CURRENT_USER\Software\Microsoft\office\16.0\Outlook\Preferences]
"NewOutlookMigrationUserSetting"=dword:00000000
"UseNewOutlook"=dword:00000000

[HKEY_CURRENT_USER\Software\Microsoft\office\16.0\Outlook\Options\General]
"DoNewOutlookAutoMigration"=dword:00000000
"NewOutlookAutoMigrationRetryIntervals"=dword:00000000
"HideNewOutlookToggle"=dword:00000001
```

### Configure & sign in to applications
- [ ] Edge -> Sign in M365 user
- [ ] Teams -> Sign in M365 user
- [ ] Outlook Classic -> Sign in M365 user
- [ ] Disable Focused Inbox
- [ ] Test email send & receive
- [ ] Install Zoom
- [ ] Install Webex
- [ ] Remove Webex from startup
- [ ] Remove Microsoft Edge from taskbar
- [ ] Pin Google Chrome to taskbar
- [ ] Set Google Chrome as default application

### Configure desktop icons
- [ ] Start -> Settings -> Personalization -> Themes -> Desktop icon settings (see screenshot):

![Fig. 5 - Desktop icon settings](./images/image5.png)

- [ ] Only show the following icons on the desktop (see screenshot):

![Fig. 6 - Desktop icons](./images/image6.png)

### Start menu power button & folders
- [ ] Start menu -> Settings -> Personalization -> Start -> Folders
- [ ] Enable all except Music and Video (see screenshot):

![Fig. 7 - Start menu folder settings](./images/image7.png)

- [ ] Enable power button icons - Start menu should look like this afterwards:

![Fig. 8 - Start menu power button](./images/image8.png)

### Pinned programs in Start menu
- [ ] Arrange pinned programs exactly according to guidelines (see screenshot):

![Fig. 9 - Start menu pinned programs](./images/image9.png)

### Configure taskbar
- [ ] Configure taskbar according to guidelines (see screenshot):

![Fig. 10 - Taskbar settings](./images/image10.png)

## 10 | Handover & Warranty Check

- [ ] Open warranty check: [pcsupport.lenovo.com](https://pcsupport.lenovo.com/us/en/warranty-lookup)
- [ ] Enter serial number and read warranty data
- [ ] Enter values into handover document (similar to figure, see screenshot):

![Fig. 11 - Lenovo warranty overview](./images/image11.png)

- [ ] Completely fill out handover document (2026-MM-DD_Protocol_Lastname,Firstname_(Notebook).docx)

![Fig. 12 - Example notebook protocol](./images/image12.png)

- [ ] Print handover document and include with notebook
