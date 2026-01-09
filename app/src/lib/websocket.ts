/**
 * WebSocket-Verbindung zum AudioMultiverse-Server
 */

import { writable, type Writable } from 'svelte/store';
import { sendFaderFeedback, sendMuteFeedback, sendSoloFeedback, updateChannelFeedback } from './midi';

/** Verbindungsstatus */
export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

/** Channel-State vom Server */
export interface ChannelState {
    id: number;
    name: string;
    fader: number;
    mute: boolean;
    solo: boolean;
    pan: number;
    gain: number;
    phase_invert: boolean;
    color: string;
    meter: number;
}

/** Mixer-State */
export interface MixerState {
    channels: ChannelState[];
    routing: number[][];
    input_count: number;
    output_count: number;
}

/** Server-Info */
export interface ServerInfo {
    name: string;
    version: string;
    input_channels: number;
    output_channels: number;
    sample_rate: number;
    uptime_seconds: number;
    connected_clients: number;
}

/** Stores für reaktive Daten */
export const connectionStatus: Writable<ConnectionStatus> = writable('disconnected');
export const serverInfo: Writable<ServerInfo | null> = writable(null);
export const mixerState: Writable<MixerState | null> = writable(null);
export const meterData: Writable<{ inputs: number[]; outputs: number[]; master: number[] } | null> = writable(null);

/** WebSocket-Verbindung */
let socket: WebSocket | null = null;
let reconnectTimer: number | null = null;
let pingInterval: number | null = null;

/** Server-URL */
let currentServerUrl: string | null = null;

/**
 * Mit Server verbinden
 */
export function connect(url: string): void {
    currentServerUrl = url;
    
    if (socket) {
        socket.close();
    }
    
    connectionStatus.set('connecting');
    
    // WebSocket URL konvertieren
    const wsUrl = url.replace(/^http/, 'ws') + '/ws';
    console.log('Connecting to:', wsUrl);
    
    try {
        socket = new WebSocket(wsUrl);
        
        socket.onopen = () => {
            console.log('WebSocket connected');
            connectionStatus.set('connected');
            
            // Hello senden
            send({
                type: 'hello',
                payload: {
                    name: 'AudioMultiverse Client',
                    client_type: 'remote',
                    version: '0.1.0'
                }
            });
            
            // Ping-Interval starten
            if (pingInterval) clearInterval(pingInterval);
            pingInterval = window.setInterval(() => {
                send({ type: 'ping', payload: { timestamp: Date.now() } });
            }, 30000);
        };
        
        socket.onclose = () => {
            console.log('WebSocket disconnected');
            connectionStatus.set('disconnected');
            
            if (pingInterval) {
                clearInterval(pingInterval);
                pingInterval = null;
            }
            
            // Auto-Reconnect
            if (currentServerUrl && !reconnectTimer) {
                reconnectTimer = window.setTimeout(() => {
                    reconnectTimer = null;
                    if (currentServerUrl) {
                        connect(currentServerUrl);
                    }
                }, 3000);
            }
        };
        
        socket.onerror = (error) => {
            console.error('WebSocket error:', error);
            connectionStatus.set('error');
        };
        
        socket.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                handleMessage(message);
            } catch (e) {
                console.error('Failed to parse message:', e);
            }
        };
    } catch (e) {
        console.error('Failed to create WebSocket:', e);
        connectionStatus.set('error');
    }
}

/**
 * Verbindung trennen
 */
export function disconnect(): void {
    currentServerUrl = null;
    
    if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
    }
    
    if (pingInterval) {
        clearInterval(pingInterval);
        pingInterval = null;
    }
    
    if (socket) {
        socket.close();
        socket = null;
    }
    
    connectionStatus.set('disconnected');
}

/**
 * Nachricht senden
 */
function send(message: object): void {
    if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify(message));
    }
}

/**
 * Eingehende Nachricht verarbeiten
 */
function handleMessage(message: { type: string; payload?: any }): void {
    switch (message.type) {
        case 'welcome':
            serverInfo.set(message.payload.server_info);
            mixerState.set(message.payload.state);
            
            // Meter-Updates anfordern
            send({
                type: 'subscribe_meters',
                payload: { enabled: true, interval_ms: 50 }
            });
            
            // Initial MIDI-Feedback senden
            sendInitialMidiFeedback(message.payload.state);
            break;
            
        case 'pong':
            // Latenz berechnen
            const latency = Date.now() - message.payload.timestamp;
            console.debug(`Server latency: ${latency}ms`);
            break;
            
        case 'channel_updated':
            updateChannel(message.payload);
            break;
            
        case 'state':
            mixerState.set(message.payload);
            break;
            
        case 'meters':
            meterData.set(message.payload);
            break;
            
        case 'error':
            console.error('Server error:', message.payload.message);
            break;
            
        default:
            console.debug('Unknown message type:', message.type);
    }
}

/**
 * Einzelnen Channel updaten
 */
function updateChannel(channel: ChannelState): void {
    mixerState.update(state => {
        if (!state) return state;
        
        const idx = state.channels.findIndex(c => c.id === channel.id);
        if (idx >= 0) {
            state.channels[idx] = channel;
        }
        
        return { ...state };
    });
    
    // MIDI Feedback senden
    updateChannelFeedback(
        channel.id,
        channel.fader,
        channel.mute,
        channel.solo,
        channel.pan,
        channel.name
    ).catch(e => console.warn('MIDI feedback error:', e));
}

/**
 * Initial MIDI-Feedback für alle Channels senden
 */
async function sendInitialMidiFeedback(state: MixerState): Promise<void> {
    for (const channel of state.channels.slice(0, 8)) { // Nur erste 8 für MCU
        try {
            await updateChannelFeedback(
                channel.id,
                channel.fader,
                channel.mute,
                channel.solo,
                channel.pan,
                channel.name
            );
        } catch (e) {
            // Ignore - might not have MIDI connected
        }
    }
}

// ============ Client -> Server Commands ============

/**
 * Fader setzen
 */
export function setFader(channel: number, value: number): void {
    send({
        type: 'set_fader',
        payload: { channel, value }
    });
}

/**
 * Mute setzen
 */
export function setMute(channel: number, muted: boolean): void {
    send({
        type: 'set_mute',
        payload: { channel, muted }
    });
}

/**
 * Solo setzen
 */
export function setSolo(channel: number, solo: boolean): void {
    send({
        type: 'set_solo',
        payload: { channel, solo }
    });
}

/**
 * Pan setzen
 */
export function setPan(channel: number, value: number): void {
    send({
        type: 'set_pan',
        payload: { channel, value }
    });
}

/**
 * Gain setzen
 */
export function setGain(channel: number, value: number): void {
    send({
        type: 'set_gain',
        payload: { channel, value }
    });
}

/**
 * Channel-Name setzen
 */
export function setChannelName(channel: number, name: string): void {
    send({
        type: 'set_channel_name',
        payload: { channel, name }
    });
}

/**
 * Routing setzen
 */
export function setRouting(input: number, output: number, gain: number): void {
    send({
        type: 'set_routing',
        payload: { input, output, gain }
    });
}

/**
 * Kompletten State anfordern
 */
export function requestState(): void {
    send({ type: 'get_state', payload: null });
}

/**
 * Server-Info anfordern
 */
export function requestServerInfo(): void {
    send({ type: 'get_server_info', payload: null });
}
