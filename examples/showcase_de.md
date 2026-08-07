---
title: "Doc2Flow Deutscher Showcase"
subtitle: "Umfassendes Testdokument für alle aktuellen und zukünftigen Funktionen"
date: "25.07.2026"
version: "1.0.0"
language: "de"
numbered_sections: true
---

[Variables]
| Variable | Value |
| --- | --- |
| SERVER_NAME | prod-srv-de-01 |
| PORT | 8080 |
| API_KEY | secret-key-de-12345 |

# Teil 1: Systemeinrichtung & Vorbereitung

Dieser übergeordnete Abschnitt beschreibt die grundlegende Systemkonfiguration. Aufgaben in H1-Abschnitten besitzen keinen eigenen Badge-Indikator, fließen aber voll in die Gesamtfortschrittsanzeige ein.

- [x] Allgemeine Sicherheitsunterweisung für Techniker durchgeführt

## Abschnitt 1: Übersicht & Richtlinien

Dies ist ein beliebiger Textabsatz im Abschnitts-Hauptteil. Er liefert allgemeine Anweisungen und Informationen für den Bearbeiter, die vor Beginn des Verfahrens durchgelesen werden sollten.

![Beispiel Systemdiagramm](../resources/images/example1.jpg)

![Externes Remote-Bild](https://picsum.photos/600/300)

<!-- Test-Kommentar: Dieser Hinweis darf nicht im HTML erscheinen -->

> Dies ist eine neutrale Hinweis-Box mit Standard-Kontext.

>? Dies ist eine grüne Tipp-Box mit Empfehlungen oder Best Practices.

>! Dies ist eine violette Wichtig-Box, die auf kritische Anforderungen hinweist.

>!! Dies ist eine gelbe Warnung-Box, die zur Vorsicht bei potenziellen Problemen rät.

>!!! Dies ist eine rote Achtung-Box, die vor gefährlichen Aktionen oder Datenverlust warnt.

> Dies ist eine mehrzeilige Hinweis-Box mit detaillierten Hintergrundinformationen und umfassenden Anweisungen für den Endbenutzer. Sie erstreckt sich über mehrere Sätze, um einen flüssigen Textumbruch und eine saubere Ausrichtung in visuellen Hinweisfeldern zu demonstrieren.

## Abschnitt 2: Aufgaben-Checkliste

### Einrichtung & Vorbereitung
Dieser beliebige Textabsatz beschreibt die vorbereitenden Schritte, die erfüllt sein müssen, bevor einzelne Aufgaben als erledigt markiert werden.

- [ ] Hardware auspacken und Komponenten überprüfen
  - [ ] Vollständigkeit des Zubehörs kontrollieren
  - [x] Gehäuse auf Transportschäden prüfen
- [ ] Netzwerk und Stromversorgung anschließen
  1. Primäres Netzwerkkabel in Port 1 einstecken
  2. Redundantes Netzwerkkabel in Port 2 einstecken
- [x] Initialer Einschalttest abgeschlossen

[Externe Systemdokumentation](https://example.com/docs)

![Systemspezifikation PDF herunterladen](https://example.com/dateien/spezifikation.pdf)

#### Software-Konfiguration
- [ ] Neueste Systemupdates installieren
- [ ] Firewall-Regeln gemäß Unternehmensrichtlinie konfigurieren
- [x] Fernzugriffs-Dienst überprüfen

![Software-Konfiguration Screenshots](../resources/images/example2.jpg)

# Teil 2: Wartung & Systemreferenz

## Abschnitt 3: Referenz & Code-Blöcke

### System-Registrierungseinstellungen
Der folgende Konfigurationsauszug muss angewendet werden, um automatische Updates zu deaktivieren:

```ini
[HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate]
"DisableWindowsUpdateAccess"=dword:00000001
"Server"="{{SERVER_NAME}}"
```

### Unbeschrifteter Skript-Block
```
echo "Doc2Flow Showcase-Umgebung {{SERVER_NAME}}:{{PORT}} mit Schlüssel {{API_KEY}} wird initialisiert..."
```

### Systemkomponenten & Spezifikationen

Die folgende Tabelle enthält die Spezifikationen der installierten Hardwarekomponenten:

| Komponente | Modell | Status | Kapazität |
|---|---|---|---|
| Hauptprozessor | Intel Xeon E-2388G | Aktiv | 8 Kerne / 16 Threads |
| Arbeitsspeicher | DDR4 ECC Registered | Optimal | 64 GB (2x 32 GB) |
| Primärspeicher | NVMe SSD PCIe 4.0 | Normal | 2 TB RAID 1 |
| Netzwerkschnittstelle | Dual 10GbE SFP+ | Verbunden | 10 Gbps |

## Abschnitt 4: Informationslisten

- Hauptaufgabe & Systemüberwachung
  1. Unterpunkt A 1: Dienststatus abfragen
  2. Unterpunkt A 2: Fehlerprotokoll analysieren
- Vorgehensweise bei Wartungsarbeiten
  - [ ] Testlauf vorbereiten
     - Detailprüfung Parameter X
     - Detailprüfung Parameter Y
  - [x] Abnahme durch Administrator

1. Sequenzieller Hauptschritt 1
   - Untergeordneter Prüfschritt 1.1
   - Untergeordneter Prüfschritt 1.2
2. Sequenzieller Hauptschritt 2
   1. Detaillierter Teilverlauf 2.a
   2. Detaillierter Teilverlauf 2.b
3. Sequenzieller Hauptschritt 3

---

![System-Informationsübersicht](../resources/images/example3.jpg)

![Weißes Testbild mit Rahmen](../resources/images/example4.png)

# Leerer Hauptabschnitt

## Leerer Unterabschnitt
