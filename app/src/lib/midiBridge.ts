/**
 * MIDI Bridge - Verbindet MIDI Controller Events mit WebSocket-Befehlen
 * 
 * Diese Bridge lauscht auf MIDI-Events vom Controller und sendet entsprechende
 * Befehle an den Server über WebSocket.
 */

import { invoke } from '@tauri-apps/api/tauri';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { setFader, setMute, setSolo, setPan } from './websocket';

/** MIDI Command Types (von Rust) */
interface MidiCommand {
    type: 'set_fader' | 'set_mute' | 'set_solo' | 'set_pan' | 'transport' | 'bank_change';
    channel?: number;
    value?: number;
    muted?: boolean;
    solo?: boolean;
    command?: string;
    direction?: string;
}

/** Event Listener */
let unlistenMidi: UnlistenFn | null = null;
let pollingInterval: number | null = null;
let isRunning = false;

/**
 * MIDI Bridge starten - lauscht auf MIDI Events und sendet an Server
 */
export async function startMidiBridge(): Promise<void> {
    if (isRunning) {
        console.log('MIDI Bridge already running');
        return;
    }
    
    isRunning = true;
    console.log('Starting MIDI Bridge...');
    
    // Event Listener für Tauri-Events (wenn verfügbar)
    try {
        unlistenMidi = await listen<MidiCommand>('midi-command', (event) => {
            handleMidiCommand(event.payload);
        });
    } catch (e) {
        console.warn('Could not setup Tauri event listener:', e);
    }
    
    // Polling als Fallback (für Systeme ohne Event-Support)
    pollingInterval = window.setInterval(async () => {
        try {
            const command = await invoke<MidiCommand | null>('poll_midi_command');
            if (command) {
                handleMidiCommand(command);
            }
        } catch (e) {
            // Ignore polling errors
        }
    }, 10); // 10ms = 100Hz polling rate
}

/**
 * MIDI Bridge stoppen
 */
export function stopMidiBridge(): void {
    isRunning = false;
    
    if (unlistenMidi) {
        unlistenMidi();
        unlistenMidi = null;
    }
    
    if (pollingInterval) {
        clearInterval(pollingInterval);
        pollingInterval = null;
    }
    
    console.log('MIDI Bridge stopped');
}

/**
 * MIDI Command verarbeiten und an Server senden
 */
function handleMidiCommand(cmd: MidiCommand): void {
    console.debug('MIDI Command:', cmd);
    
    switch (cmd.type) {
        case 'set_fader':
            if (cmd.channel !== undefined && cmd.value !== undefined) {
                // MIDI value (0-127) zu 0.0-1.0 konvertieren
                const normalizedValue = cmd.value / 127.0;
                setFader(cmd.channel, normalizedValue);
            }
            break;
            
        case 'set_mute':
            if (cmd.channel !== undefined && cmd.muted !== undefined) {
                setMute(cmd.channel, cmd.muted);
            }
            break;
            
        case 'set_solo':
            if (cmd.channel !== undefined && cmd.solo !== undefined) {
                setSolo(cmd.channel, cmd.solo);
            }
            break;
            
        case 'set_pan':
            if (cmd.channel !== undefined && cmd.value !== undefined) {
                // MIDI value (0-127, center 64) zu -1.0 bis 1.0 konvertieren
                const normalizedPan = (cmd.value - 64) / 63.0;
                setPan(cmd.channel, Math.max(-1.0, Math.min(1.0, normalizedPan)));
            }
            break;
            
        case 'transport':
            // Transport commands (Play, Stop, Record, etc.)
            console.log('Transport command:', cmd.command);
            // TODO: Transport über WebSocket senden wenn implementiert
            break;
            
        case 'bank_change':
            // Bank hoch/runter
            console.log('Bank change:', cmd.direction);
            // TODO: Bank-Wechsel implementieren (Channel-Offset ändern)
            break;
            
        default:
            console.debug('Unknown MIDI command type:', cmd);
    }
}

/**
 * MIDI Learn starten für einen bestimmten Parameter
 */
export async function startMidiLearn(
    targetType: 'fader' | 'mute' | 'solo' | 'pan' | 'transport',
    targetChannel?: number
): Promise<{ channel: number; cc?: number; note?: number } | null> {
    try {
        const result = await invoke<{ channel: number; cc?: number; note?: number } | null>(
            'start_midi_learn',
            { 
                targetType, 
                channel: targetChannel ?? 0
            }
        );
        
        if (result) {
            console.log('MIDI Learn result:', result);
        }
        
        return result;
    } catch (e) {
        console.error('MIDI Learn error:', e);
        return null;
    }
}

/**
 * MIDI Learn abbrechen
 */
export async function cancelMidiLearn(): Promise<void> {
    try {
        await invoke('cancel_midi_learn');
    } catch (e) {
        console.error('Cancel MIDI Learn error:', e);
    }
}
