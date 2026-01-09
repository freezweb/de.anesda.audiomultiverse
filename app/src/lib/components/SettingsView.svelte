<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    
    const dispatch = createEventDispatcher();
    
    // Settings Kategorien
    type SettingsCategory = 'audio' | 'network' | 'midi' | 'appearance' | 'system';
    let activeCategory: SettingsCategory = 'audio';
    
    // Audio Settings
    let audioSettings = {
        inputDevice: '',
        outputDevice: '',
        sampleRate: 48000,
        bufferSize: 256,
        inputChannels: 32,
        outputChannels: 32,
    };
    
    // Verfügbare Audio-Geräte
    let inputDevices: string[] = ['System Default', 'DANTE Virtual Soundcard', 'Focusrite Scarlett 18i20'];
    let outputDevices: string[] = ['System Default', 'DANTE Virtual Soundcard', 'Focusrite Scarlett 18i20'];
    
    // Netzwerk Settings
    let networkSettings = {
        serverAddress: 'localhost',
        serverPort: 8080,
        autoConnect: true,
        reconnectInterval: 5,
        enableMdns: true,
    };
    
    // MIDI Settings
    let midiSettings = {
        enabled: true,
        inputDevices: [] as string[],
        outputDevices: [] as string[],
        mcuMode: false,
        feedbackEnabled: true,
    };
    
    // Verfügbare MIDI-Geräte
    let midiInputDevices: string[] = ['Behringer X-Touch', 'nanoKONTROL2', 'Launchpad'];
    let midiOutputDevices: string[] = ['Behringer X-Touch', 'nanoKONTROL2'];
    
    // Appearance Settings
    let appearanceSettings = {
        theme: 'dark' as 'dark' | 'light',
        accentColor: '#3b82f6',
        channelWidth: 'normal' as 'compact' | 'normal' | 'wide',
        showMeters: true,
        meterBallistics: 'peak' as 'peak' | 'rms' | 'both',
        animationsEnabled: true,
    };
    
    // System Settings
    let systemSettings = {
        scenesPath: './scenes',
        autoSaveInterval: 300,
        logLevel: 'info' as 'debug' | 'info' | 'warn' | 'error',
        enableTelemetry: false,
    };
    
    // Dirty flag für unsaved changes
    let hasUnsavedChanges = false;
    
    function saveSettings() {
        dispatch('save', {
            audio: audioSettings,
            network: networkSettings,
            midi: midiSettings,
            appearance: appearanceSettings,
            system: systemSettings,
        });
        
        hasUnsavedChanges = false;
    }
    
    function resetToDefaults() {
        if (confirm('Alle Einstellungen auf Standard zurücksetzen?')) {
            dispatch('reset');
        }
    }
    
    function markDirty() {
        hasUnsavedChanges = true;
    }
    
    // Accent Farben
    const accentColors = [
        { name: 'Blue', value: '#3b82f6' },
        { name: 'Green', value: '#22c55e' },
        { name: 'Purple', value: '#a855f7' },
        { name: 'Red', value: '#ef4444' },
        { name: 'Orange', value: '#f97316' },
        { name: 'Teal', value: '#14b8a6' },
    ];
</script>

<div class="h-full flex bg-mixer-bg text-white">
    <!-- Sidebar Navigation -->
    <div class="w-64 border-r border-gray-700 p-4">
        <h2 class="text-xl font-bold mb-6">⚙️ Einstellungen</h2>
        
        <nav class="space-y-1">
            <button
                on:click={() => activeCategory = 'audio'}
                class="w-full text-left px-4 py-3 rounded-lg transition
                       {activeCategory === 'audio' ? 'bg-mixer-accent' : 'hover:bg-gray-700'}"
            >
                🔊 Audio
            </button>
            
            <button
                on:click={() => activeCategory = 'network'}
                class="w-full text-left px-4 py-3 rounded-lg transition
                       {activeCategory === 'network' ? 'bg-mixer-accent' : 'hover:bg-gray-700'}"
            >
                🌐 Netzwerk
            </button>
            
            <button
                on:click={() => activeCategory = 'midi'}
                class="w-full text-left px-4 py-3 rounded-lg transition
                       {activeCategory === 'midi' ? 'bg-mixer-accent' : 'hover:bg-gray-700'}"
            >
                🎹 MIDI
            </button>
            
            <button
                on:click={() => activeCategory = 'appearance'}
                class="w-full text-left px-4 py-3 rounded-lg transition
                       {activeCategory === 'appearance' ? 'bg-mixer-accent' : 'hover:bg-gray-700'}"
            >
                🎨 Darstellung
            </button>
            
            <button
                on:click={() => activeCategory = 'system'}
                class="w-full text-left px-4 py-3 rounded-lg transition
                       {activeCategory === 'system' ? 'bg-mixer-accent' : 'hover:bg-gray-700'}"
            >
                🖥️ System
            </button>
        </nav>
        
        <!-- Save / Reset Buttons -->
        <div class="mt-8 space-y-2">
            <button
                on:click={saveSettings}
                disabled={!hasUnsavedChanges}
                class="w-full py-2 px-4 bg-green-600 hover:bg-green-500 disabled:bg-gray-700 
                       disabled:text-gray-500 disabled:cursor-not-allowed rounded-lg font-medium transition"
            >
                💾 Speichern
            </button>
            
            <button
                on:click={resetToDefaults}
                class="w-full py-2 px-4 bg-gray-700 hover:bg-gray-600 rounded-lg transition"
            >
                ↩️ Zurücksetzen
            </button>
        </div>
        
        {#if hasUnsavedChanges}
            <p class="mt-4 text-sm text-yellow-400 text-center">
                ⚠️ Ungespeicherte Änderungen
            </p>
        {/if}
    </div>
    
    <!-- Settings Content -->
    <div class="flex-1 overflow-y-auto p-8">
        <!-- Audio Settings -->
        {#if activeCategory === 'audio'}
            <div class="max-w-2xl">
                <h3 class="text-2xl font-bold mb-6">🔊 Audio-Einstellungen</h3>
                
                <div class="space-y-6">
                    <!-- Input Device -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Eingabe-Gerät
                        </label>
                        <select
                            bind:value={audioSettings.inputDevice}
                            on:change={markDirty}
                            class="w-full px-4 py-3 bg-mixer-surface border border-gray-600 rounded-lg
                                   focus:border-mixer-accent focus:outline-none"
                        >
                            {#each inputDevices as device}
                                <option value={device}>{device}</option>
                            {/each}
                        </select>
                    </div>
                    
                    <!-- Output Device -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Ausgabe-Gerät
                        </label>
                        <select
                            bind:value={audioSettings.outputDevice}
                            on:change={markDirty}
                            class="w-full px-4 py-3 bg-mixer-surface border border-gray-600 rounded-lg
                                   focus:border-mixer-accent focus:outline-none"
                        >
                            {#each outputDevices as device}
                                <option value={device}>{device}</option>
                            {/each}
                        </select>
                    </div>
                    
                    <!-- Sample Rate -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Abtastrate
                        </label>
                        <select
                            bind:value={audioSettings.sampleRate}
                            on:change={markDirty}
                            class="w-full px-4 py-3 bg-mixer-surface border border-gray-600 rounded-lg
                                   focus:border-mixer-accent focus:outline-none"
                        >
                            <option value={44100}>44.1 kHz</option>
                            <option value={48000}>48 kHz</option>
                            <option value={88200}>88.2 kHz</option>
                            <option value={96000}>96 kHz</option>
                        </select>
                    </div>
                    
                    <!-- Buffer Size -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Puffer-Größe: {audioSettings.bufferSize} Samples
                            <span class="text-gray-500">
                                ({(audioSettings.bufferSize / audioSettings.sampleRate * 1000).toFixed(1)} ms)
                            </span>
                        </label>
                        <input
                            type="range"
                            bind:value={audioSettings.bufferSize}
                            on:input={markDirty}
                            min="64"
                            max="2048"
                            step="64"
                            class="w-full accent-mixer-accent"
                        />
                        <div class="flex justify-between text-xs text-gray-500 mt-1">
                            <span>64 (Low Latency)</span>
                            <span>2048 (Stable)</span>
                        </div>
                    </div>
                    
                    <!-- Channel Count -->
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-300 mb-2">
                                Eingangskanäle
                            </label>
                            <input
                                type="number"
                                bind:value={audioSettings.inputChannels}
                                on:input={markDirty}
                                min="2"
                                max="128"
                                step="2"
                                class="w-full px-4 py-3 bg-mixer-surface border border-gray-600 rounded-lg
                                       focus:border-mixer-accent focus:outline-none"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-300 mb-2">
                                Ausgangskanäle
                            </label>
                            <input
                                type="number"
                                bind:value={audioSettings.outputChannels}
                                on:input={markDirty}
                                min="2"
                                max="128"
                                step="2"
                                class="w-full px-4 py-3 bg-mixer-surface border border-gray-600 rounded-lg
                                       focus:border-mixer-accent focus:outline-none"
                            />
                        </div>
                    </div>
                </div>
            </div>
        {/if}
        
        <!-- Network Settings -->
        {#if activeCategory === 'network'}
            <div class="max-w-2xl">
                <h3 class="text-2xl font-bold mb-6">🌐 Netzwerk-Einstellungen</h3>
                
                <div class="space-y-6">
                    <!-- Server Address -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Server-Adresse
                        </label>
                        <input
                            type="text"
                            bind:value={networkSettings.serverAddress}
                            on:input={markDirty}
                            placeholder="localhost oder IP-Adresse"
                            class="w-full px-4 py-3 bg-mixer-surface border border-gray-600 rounded-lg
                                   focus:border-mixer-accent focus:outline-none"
                        />
                    </div>
                    
                    <!-- Server Port -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Server-Port
                        </label>
                        <input
                            type="number"
                            bind:value={networkSettings.serverPort}
                            on:input={markDirty}
                            min="1024"
                            max="65535"
                            class="w-full px-4 py-3 bg-mixer-surface border border-gray-600 rounded-lg
                                   focus:border-mixer-accent focus:outline-none"
                        />
                    </div>
                    
                    <!-- Auto Connect -->
                    <label class="flex items-center gap-3 cursor-pointer">
                        <input
                            type="checkbox"
                            bind:checked={networkSettings.autoConnect}
                            on:change={markDirty}
                            class="w-5 h-5 accent-mixer-accent"
                        />
                        <span>Automatisch verbinden beim Start</span>
                    </label>
                    
                    <!-- Reconnect Interval -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Wiederverbindungs-Intervall: {networkSettings.reconnectInterval} Sekunden
                        </label>
                        <input
                            type="range"
                            bind:value={networkSettings.reconnectInterval}
                            on:input={markDirty}
                            min="1"
                            max="30"
                            class="w-full accent-mixer-accent"
                        />
                    </div>
                    
                    <!-- mDNS -->
                    <label class="flex items-center gap-3 cursor-pointer">
                        <input
                            type="checkbox"
                            bind:checked={networkSettings.enableMdns}
                            on:change={markDirty}
                            class="w-5 h-5 accent-mixer-accent"
                        />
                        <span>mDNS/Bonjour für Server-Erkennung aktivieren</span>
                    </label>
                </div>
            </div>
        {/if}
        
        <!-- MIDI Settings -->
        {#if activeCategory === 'midi'}
            <div class="max-w-2xl">
                <h3 class="text-2xl font-bold mb-6">🎹 MIDI-Einstellungen</h3>
                
                <div class="space-y-6">
                    <!-- MIDI Enabled -->
                    <label class="flex items-center gap-3 cursor-pointer">
                        <input
                            type="checkbox"
                            bind:checked={midiSettings.enabled}
                            on:change={markDirty}
                            class="w-5 h-5 accent-mixer-accent"
                        />
                        <span class="font-medium">MIDI aktivieren</span>
                    </label>
                    
                    {#if midiSettings.enabled}
                        <!-- MIDI Input Devices -->
                        <div>
                            <label class="block text-sm font-medium text-gray-300 mb-2">
                                MIDI-Eingabegeräte
                            </label>
                            <div class="space-y-2">
                                {#each midiInputDevices as device}
                                    <label class="flex items-center gap-3 p-3 bg-mixer-surface rounded-lg cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={midiSettings.inputDevices.includes(device)}
                                            on:change={(e) => {
                                                if (e.currentTarget.checked) {
                                                    midiSettings.inputDevices = [...midiSettings.inputDevices, device];
                                                } else {
                                                    midiSettings.inputDevices = midiSettings.inputDevices.filter(d => d !== device);
                                                }
                                                markDirty();
                                            }}
                                            class="w-5 h-5 accent-mixer-accent"
                                        />
                                        <span>{device}</span>
                                    </label>
                                {/each}
                            </div>
                        </div>
                        
                        <!-- MIDI Output Devices -->
                        <div>
                            <label class="block text-sm font-medium text-gray-300 mb-2">
                                MIDI-Ausgabegeräte (für Feedback)
                            </label>
                            <div class="space-y-2">
                                {#each midiOutputDevices as device}
                                    <label class="flex items-center gap-3 p-3 bg-mixer-surface rounded-lg cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={midiSettings.outputDevices.includes(device)}
                                            on:change={(e) => {
                                                if (e.currentTarget.checked) {
                                                    midiSettings.outputDevices = [...midiSettings.outputDevices, device];
                                                } else {
                                                    midiSettings.outputDevices = midiSettings.outputDevices.filter(d => d !== device);
                                                }
                                                markDirty();
                                            }}
                                            class="w-5 h-5 accent-mixer-accent"
                                        />
                                        <span>{device}</span>
                                    </label>
                                {/each}
                            </div>
                        </div>
                        
                        <!-- MCU Mode -->
                        <label class="flex items-center gap-3 cursor-pointer">
                            <input
                                type="checkbox"
                                bind:checked={midiSettings.mcuMode}
                                on:change={markDirty}
                                class="w-5 h-5 accent-mixer-accent"
                            />
                            <span>Mackie Control (MCU) Modus</span>
                        </label>
                        
                        <!-- Feedback -->
                        <label class="flex items-center gap-3 cursor-pointer">
                            <input
                                type="checkbox"
                                bind:checked={midiSettings.feedbackEnabled}
                                on:change={markDirty}
                                class="w-5 h-5 accent-mixer-accent"
                            />
                            <span>MIDI-Feedback aktivieren (motorisierte Fader, LEDs)</span>
                        </label>
                        
                        <!-- MIDI Learn Button -->
                        <button
                            class="w-full py-3 px-4 bg-purple-600 hover:bg-purple-500 rounded-lg font-medium"
                        >
                            🎓 MIDI-Learn öffnen
                        </button>
                    {/if}
                </div>
            </div>
        {/if}
        
        <!-- Appearance Settings -->
        {#if activeCategory === 'appearance'}
            <div class="max-w-2xl">
                <h3 class="text-2xl font-bold mb-6">🎨 Darstellung</h3>
                
                <div class="space-y-6">
                    <!-- Theme -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Farbschema
                        </label>
                        <div class="flex gap-4">
                            <button
                                on:click={() => { appearanceSettings.theme = 'dark'; markDirty(); }}
                                class="flex-1 py-4 rounded-lg border-2 transition
                                       {appearanceSettings.theme === 'dark' 
                                           ? 'border-mixer-accent bg-gray-900' 
                                           : 'border-gray-600 bg-gray-800'}"
                            >
                                🌙 Dunkel
                            </button>
                            <button
                                on:click={() => { appearanceSettings.theme = 'light'; markDirty(); }}
                                class="flex-1 py-4 rounded-lg border-2 transition
                                       {appearanceSettings.theme === 'light' 
                                           ? 'border-mixer-accent bg-gray-200 text-gray-900' 
                                           : 'border-gray-600 bg-gray-800'}"
                            >
                                ☀️ Hell
                            </button>
                        </div>
                    </div>
                    
                    <!-- Accent Color -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Akzentfarbe
                        </label>
                        <div class="flex gap-3">
                            {#each accentColors as color}
                                <button
                                    on:click={() => { appearanceSettings.accentColor = color.value; markDirty(); }}
                                    title={color.name}
                                    class="w-10 h-10 rounded-lg transition
                                           {appearanceSettings.accentColor === color.value 
                                               ? 'ring-2 ring-white ring-offset-2 ring-offset-mixer-bg' 
                                               : ''}"
                                    style="background-color: {color.value}"
                                />
                            {/each}
                        </div>
                    </div>
                    
                    <!-- Channel Width -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Kanalbreite
                        </label>
                        <select
                            bind:value={appearanceSettings.channelWidth}
                            on:change={markDirty}
                            class="w-full px-4 py-3 bg-mixer-surface border border-gray-600 rounded-lg
                                   focus:border-mixer-accent focus:outline-none"
                        >
                            <option value="compact">Kompakt</option>
                            <option value="normal">Normal</option>
                            <option value="wide">Breit</option>
                        </select>
                    </div>
                    
                    <!-- Meter Settings -->
                    <label class="flex items-center gap-3 cursor-pointer">
                        <input
                            type="checkbox"
                            bind:checked={appearanceSettings.showMeters}
                            on:change={markDirty}
                            class="w-5 h-5 accent-mixer-accent"
                        />
                        <span>Pegelanzeigen anzeigen</span>
                    </label>
                    
                    {#if appearanceSettings.showMeters}
                        <div>
                            <label class="block text-sm font-medium text-gray-300 mb-2">
                                Meter-Modus
                            </label>
                            <select
                                bind:value={appearanceSettings.meterBallistics}
                                on:change={markDirty}
                                class="w-full px-4 py-3 bg-mixer-surface border border-gray-600 rounded-lg
                                       focus:border-mixer-accent focus:outline-none"
                            >
                                <option value="peak">Peak</option>
                                <option value="rms">RMS</option>
                                <option value="both">Peak + RMS</option>
                            </select>
                        </div>
                    {/if}
                    
                    <!-- Animations -->
                    <label class="flex items-center gap-3 cursor-pointer">
                        <input
                            type="checkbox"
                            bind:checked={appearanceSettings.animationsEnabled}
                            on:change={markDirty}
                            class="w-5 h-5 accent-mixer-accent"
                        />
                        <span>Animationen aktivieren</span>
                    </label>
                </div>
            </div>
        {/if}
        
        <!-- System Settings -->
        {#if activeCategory === 'system'}
            <div class="max-w-2xl">
                <h3 class="text-2xl font-bold mb-6">🖥️ System</h3>
                
                <div class="space-y-6">
                    <!-- Scenes Path -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Szenen-Speicherort
                        </label>
                        <input
                            type="text"
                            bind:value={systemSettings.scenesPath}
                            on:input={markDirty}
                            class="w-full px-4 py-3 bg-mixer-surface border border-gray-600 rounded-lg
                                   focus:border-mixer-accent focus:outline-none"
                        />
                    </div>
                    
                    <!-- Auto Save -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Auto-Save Intervall: {systemSettings.autoSaveInterval} Sekunden
                        </label>
                        <input
                            type="range"
                            bind:value={systemSettings.autoSaveInterval}
                            on:input={markDirty}
                            min="60"
                            max="600"
                            step="30"
                            class="w-full accent-mixer-accent"
                        />
                        <div class="flex justify-between text-xs text-gray-500 mt-1">
                            <span>1 Minute</span>
                            <span>10 Minuten</span>
                        </div>
                    </div>
                    
                    <!-- Log Level -->
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            Log-Level
                        </label>
                        <select
                            bind:value={systemSettings.logLevel}
                            on:change={markDirty}
                            class="w-full px-4 py-3 bg-mixer-surface border border-gray-600 rounded-lg
                                   focus:border-mixer-accent focus:outline-none"
                        >
                            <option value="debug">Debug</option>
                            <option value="info">Info</option>
                            <option value="warn">Warnung</option>
                            <option value="error">Fehler</option>
                        </select>
                    </div>
                    
                    <!-- Telemetry -->
                    <label class="flex items-center gap-3 cursor-pointer">
                        <input
                            type="checkbox"
                            bind:checked={systemSettings.enableTelemetry}
                            on:change={markDirty}
                            class="w-5 h-5 accent-mixer-accent"
                        />
                        <span>Anonyme Nutzungsstatistiken senden</span>
                    </label>
                    
                    <!-- System Info -->
                    <div class="mt-8 p-4 bg-mixer-surface rounded-lg">
                        <h4 class="font-bold mb-3">System-Information</h4>
                        <div class="space-y-1 text-sm text-gray-400">
                            <p>Version: <span class="text-white">0.1.0</span></p>
                            <p>Server: <span class="text-white">AudioMultiverse Server 0.1.0</span></p>
                            <p>Protokoll: <span class="text-white">WebSocket v1</span></p>
                        </div>
                    </div>
                </div>
            </div>
        {/if}
    </div>
</div>
