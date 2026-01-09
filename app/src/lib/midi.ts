/**
 * MIDI-Integration für AudioMultiverse
 * 
 * Wrapper um Tauri-Commands für MIDI-Controller-Steuerung
 */

import { invoke } from '@tauri-apps/api/tauri';

/**
 * Liste verfügbare MIDI-Eingabegeräte
 */
export async function listMidiInputs(): Promise<string[]> {
    return await invoke('list_midi_inputs');
}

/**
 * Liste verfügbare MIDI-Ausgabegeräte
 */
export async function listMidiOutputs(): Promise<string[]> {
    return await invoke('list_midi_outputs');
}

/**
 * MIDI-Eingabegerät verbinden
 */
export async function connectMidiInput(deviceName: string): Promise<void> {
    await invoke('connect_midi_input', { deviceName });
}

/**
 * MIDI-Ausgabegerät verbinden
 */
export async function connectMidiOutput(deviceName: string): Promise<void> {
    await invoke('connect_midi_output', { deviceName });
}

/**
 * MCU-Modus aktivieren (Mackie Control Universal)
 */
export async function enableMcuMode(): Promise<void> {
    await invoke('enable_mcu_mode');
}

/**
 * MIDI Learn starten
 */
export async function startMidiLearn(
    targetType: 'fader' | 'mute' | 'solo' | 'pan' | 'master',
    channel?: number
): Promise<void> {
    await invoke('start_midi_learn', { targetType, channel });
}

/**
 * MIDI Learn abbrechen
 */
export async function cancelMidiLearn(): Promise<void> {
    await invoke('cancel_midi_learn');
}

/**
 * Fader-Feedback senden (für Motorisierte Fader)
 */
export async function sendFaderFeedback(channel: number, value: number): Promise<void> {
    await invoke('send_fader_feedback', { channel, value });
}

/**
 * Mute-Feedback senden (LED)
 */
export async function sendMuteFeedback(channel: number, muted: boolean): Promise<void> {
    await invoke('send_mute_feedback', { channel, muted });
}

/**
 * Solo-Feedback senden (LED)
 */
export async function sendSoloFeedback(channel: number, solo: boolean): Promise<void> {
    await invoke('send_solo_feedback', { channel, solo });
}

/**
 * Kompletten Channel-State als Feedback senden
 */
export async function updateChannelFeedback(
    channel: number,
    fader: number,
    muted: boolean,
    solo: boolean,
    pan: number,
    name: string
): Promise<void> {
    await invoke('update_channel_feedback', {
        channel,
        fader,
        muted,
        solo,
        pan,
        name
    });
}

/**
 * Server-URL setzen
 */
export async function setServerUrl(url: string): Promise<void> {
    await invoke('set_server_url', { url });
}

/**
 * Server-URL abrufen
 */
export async function getServerUrl(): Promise<string | null> {
    return await invoke('get_server_url');
}

/**
 * MIDI-Konfiguration
 */
export interface MidiConfig {
    inputDevice: string | null;
    outputDevice: string | null;
    mcuMode: boolean;
}

/**
 * MIDI-Manager Klasse für einfachere Verwendung
 */
export class MidiManager {
    private config: MidiConfig = {
        inputDevice: null,
        outputDevice: null,
        mcuMode: false
    };

    /**
     * Initialisiere MIDI-System
     */
    async initialize(): Promise<void> {
        // Geräte scannen
        const inputs = await listMidiInputs();
        const outputs = await listMidiOutputs();
        
        console.log('MIDI Inputs:', inputs);
        console.log('MIDI Outputs:', outputs);
    }

    /**
     * Mit einem Gerät verbinden (versucht Input und Output)
     */
    async connectDevice(deviceName: string): Promise<void> {
        try {
            await connectMidiInput(deviceName);
            this.config.inputDevice = deviceName;
            console.log(`MIDI Input connected: ${deviceName}`);
        } catch (e) {
            console.warn(`Could not connect MIDI input: ${e}`);
        }

        try {
            // Output hat oft gleichen oder ähnlichen Namen
            await connectMidiOutput(deviceName);
            this.config.outputDevice = deviceName;
            console.log(`MIDI Output connected: ${deviceName}`);
        } catch (e) {
            console.warn(`Could not connect MIDI output: ${e}`);
        }
    }

    /**
     * MCU-Modus aktivieren
     */
    async enableMcu(): Promise<void> {
        await enableMcuMode();
        this.config.mcuMode = true;
    }

    /**
     * Aktuelle Konfiguration
     */
    getConfig(): MidiConfig {
        return { ...this.config };
    }
}

// Singleton-Instanz
export const midiManager = new MidiManager();
