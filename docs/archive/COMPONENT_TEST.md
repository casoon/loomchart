# Component Test Guide

## Test-Schritte im Browser

Server läuft auf: **http://localhost:4323**

### 1. Überprüfe, ob Buttons sichtbar sind

Nach dem Laden der Seite solltest du folgende neue Buttons in der Toolbar sehen:

- **[+] Indicators** Button
- **[Panels Icon] Panels** Button  
- **[Database Icon] Cache** Button

### 2. Test Indicator Selector

**Test:**
1. Klick auf den **"Indicators"** Button
2. Ein Modal sollte erscheinen mit kategorisierten Indikatoren
3. Suche nach "RSI" im Suchfeld
4. Nur RSI-Indikatoren sollten sichtbar bleiben

**Erwartetes Verhalten:**
- Modal öffnet sich
- Kategorien: Trend, Momentum, Volume, Volatility
- Jeder Indikator hat "Overlay" und/oder "Panel" Buttons
- Suche funktioniert
- ESC schließt Modal

**Browser-Konsole Test:**
```javascript
// Modal öffnen
window.dispatchEvent(new CustomEvent('open-indicator-selector'));

// Funktion direkt aufrufen
window.showIndicatorSelector();
```

### 3. Test Panel Manager

**Test:**
1. Klick auf den **"Panels"** Button
2. Sidebar von rechts sollte erscheinen
3. Sollte "Main Chart" Panel zeigen

**Erwartetes Verhalten:**
- Sidebar öffnet sich von rechts
- Zeigt aktive Panels
- "Add Indicator" Button öffnet Indicator Selector
- "Reset Layout" Button funktioniert

**Browser-Konsole Test:**
```javascript
// Panel Manager öffnen
window.dispatchEvent(new CustomEvent('toggle-panel-manager'));

// Funktion direkt aufrufen
window.showPanelManager();
```

### 4. Test Cache Manager

**Test:**
1. Klick auf den **"Cache"** Button
2. Modal mit Cache-Statistiken sollte erscheinen
3. Zeigt 4 Statistik-Karten

**Erwartetes Verhalten:**
- Modal öffnet sich
- Zeigt: Total Entries, Total Size, Oldest/Newest Entry
- "Clear Old Entries" und "Clear All Cache" Buttons funktionieren

**Browser-Konsole Test:**
```javascript
// Cache Manager öffnen
window.dispatchEvent(new CustomEvent('toggle-cache-manager'));

// Funktion direkt aufrufen
window.showCacheManager();
```

### 5. Test Chart Axes (fortgeschritten)

**Test:**
1. Wenn Chart geladen ist, sollte rechts eine Y-Achse erscheinen
2. Unten sollte eine X-Achse erscheinen
3. Rechtsklick auf Y-Achse sollte Kontext-Menü zeigen

**Erwartetes Verhalten:**
- Achsen sind sichtbar als Overlays
- Kontext-Menü mit Optionen: Auto Scale, Reset Scale, Log Scale, etc.
- Drag auf Achse sollte skalieren (funktioniert erst mit echten Daten)

**Browser-Konsole Test:**
```javascript
// Zugriff auf Achsen-Manager
window.chartAxisManager;

// Scale ändern
window.chartAxisManager.updateYScale(39000, 43000);
```

### 6. Test Chart Pan

**Test:**
1. Bewege Maus über Chart-Bereich
2. Cursor sollte zu "grab" wechseln
3. Click-Drag sollte Pan simulieren

**Erwartetes Verhalten:**
- Cursor ändert sich zu "grab"
- Beim Drag zu "grabbing"
- Scroll-Rad scrollt horizontal (KEIN Zoom)
- Tastatur: ← → ↑ ↓ funktionieren

**Browser-Konsole Test:**
```javascript
// Zugriff auf Pan-Manager
window.chartPanManager;

// Pan aktivieren/deaktivieren
window.chartPanManager.setEnabled(false);
window.chartPanManager.setEnabled(true);

// Viewport info
window.chartPanManager.getViewport();
```

## Debug-Hilfen

### Alle verfügbaren globalen Funktionen prüfen

```javascript
// Im Browser Console
console.log('WASM:', window.wasmModule);
console.log('Show Indicator Selector:', window.showIndicatorSelector);
console.log('Show Panel Manager:', window.showPanelManager);
console.log('Show Cache Manager:', window.showCacheManager);
console.log('Refresh Panels:', window.refreshPanels);
console.log('Chart Axis Manager:', window.chartAxisManager);
console.log('Chart Pan Manager:', window.chartPanManager);
```

### Event-Listener testen

```javascript
// Test alle Custom Events
window.dispatchEvent(new CustomEvent('open-indicator-selector'));
window.dispatchEvent(new CustomEvent('toggle-panel-manager'));
window.dispatchEvent(new CustomEvent('toggle-cache-manager'));
```

### WASM-Funktionen testen (wenn verfügbar)

```javascript
// Warten bis WASM geladen ist
window.addEventListener('wasmReady', () => {
  console.log('WASM is ready!');
  
  // Panel Layout abrufen
  window.wasmModule.get_panel_layout().then(layout => {
    console.log('Current layout:', JSON.parse(layout));
  });
});
```

## Häufige Probleme

### Problem: Buttons sichtbar aber nichts passiert

**Lösung:**
```javascript
// Prüfe ob Event Listener registriert sind
window.showIndicatorSelector; // sollte Function sein
window.showPanelManager;      // sollte Function sein
window.showCacheManager;      // sollte Function sein

// Falls undefined, Seite neu laden (Ctrl+Shift+R für Hard Reload)
```

### Problem: "WASM not initialized" Fehler

**Ursache:** WASM-Modul wurde noch nicht geladen

**Lösung:**
1. Warte 1-2 Sekunden nach Seitenladen
2. Prüfe `window.wasmModule` in Console
3. Falls undefined, prüfe Browser Console für WASM-Ladefehler

### Problem: Modals erscheinen nicht

**Lösung:**
```javascript
// Prüfe ob Modal-Elemente existieren
document.getElementById('indicator-selector');  // sollte <div> sein
document.getElementById('panel-manager');       // sollte <div> sein
document.getElementById('cache-manager');       // sollte <div> sein

// Prüfe "hidden" class
document.getElementById('indicator-selector').classList.contains('hidden'); // sollte true sein
```

### Problem: IndexedDB Fehler

**Lösung:**
```javascript
// Prüfe ob IndexedDB verfügbar ist
if (!window.indexedDB) {
  console.error('IndexedDB not supported');
}

// Cache DB manuell initialisieren
import { candleCacheDB } from './lib/candle-cache-db';
await candleCacheDB.init();
```

## Erfolgreiche Tests

Wenn alles funktioniert, solltest du:

✅ Alle 3 Toolbar-Buttons sehen und anklicken können  
✅ Indicator Selector Modal öffnen können  
✅ Panel Manager Sidebar öffnen können  
✅ Cache Manager Modal öffnen können  
✅ Keine JavaScript-Fehler in Console sehen  
✅ Chart-Bereich zeigt "grab" Cursor  
✅ Custom Events funktionieren in Console  

## Nächste Schritte

Nach erfolgreichen Tests:

1. **Indikatoren hinzufügen testen:**
   - Wähle RSI → "Add as Panel"
   - Sollte neuen Panel unter Chart erstellen (braucht WASM)

2. **Panel-Verwaltung testen:**
   - Panel Manager öffnen
   - Panel ein-/ausblenden
   - Panel entfernen

3. **Cache testen:**
   - Layout speichern
   - Layout laden
   - Cache leeren

4. **Achsen-Interaktion testen:**
   - Drag auf Y-Achse → Scale ändern
   - Rechtsklick → Kontext-Menü
   - Reset Scale

5. **Pan/Scroll testen:**
   - Scroll-Rad → horizontal Pan
   - Click-Drag → Pan
   - Pfeiltasten → Pan
