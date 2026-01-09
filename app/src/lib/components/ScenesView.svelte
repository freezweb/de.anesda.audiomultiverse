<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    
    const dispatch = createEventDispatcher();
    
    interface Scene {
        id: string;
        name: string;
        description?: string;
        created_at: string;
        modified_at: string;
        color?: string;
        category?: string;
        author?: string;
    }
    
    // Szenen-Liste (wird vom Store geladen)
    export let scenes: Scene[] = [
        { id: '1', name: 'Standard', description: 'Grundeinstellung', created_at: '2025-01-01', modified_at: '2025-01-01', category: 'Vorlagen' },
        { id: '2', name: 'Live Band', description: 'Rock/Pop Live Setup', created_at: '2025-01-05', modified_at: '2025-01-07', category: 'Live', color: '#e74c3c' },
        { id: '3', name: 'Podcast', description: '4 Sprecher Setup', created_at: '2025-01-02', modified_at: '2025-01-06', category: 'Produktion', color: '#3498db' },
        { id: '4', name: 'Theater', description: 'Sprechtheater', created_at: '2025-01-03', modified_at: '2025-01-03', category: 'Live', color: '#9b59b6' },
    ];
    
    // Aktive Szene
    export let currentSceneId: string | null = null;
    
    // UI State
    let selectedSceneId: string | null = null;
    let editMode = false;
    let showNewDialog = false;
    let showRecallOptions = false;
    
    // Neue Szene Dialog
    let newSceneName = '';
    let newSceneDescription = '';
    let newSceneCategory = '';
    let newSceneColor = '#27ae60';
    
    // Recall Filter
    let recallFilter = {
        faders: true,
        mutes: true,
        solos: true,
        pans: true,
        eq: true,
        routing: true,
        names: false,
    };
    
    // Kategorien extrahieren
    $: categories = [...new Set(scenes.map(s => s.category).filter(Boolean))] as string[];
    
    // Szenen nach Kategorie gruppieren
    $: groupedScenes = categories.reduce((acc, cat) => {
        acc[cat] = scenes.filter(s => s.category === cat);
        return acc;
    }, {} as Record<string, Scene[]>);
    
    // Szenen ohne Kategorie
    $: uncategorizedScenes = scenes.filter(s => !s.category);
    
    function selectScene(id: string) {
        selectedSceneId = id;
        editMode = false;
    }
    
    function recallScene() {
        if (!selectedSceneId) return;
        
        dispatch('recall', {
            sceneId: selectedSceneId,
            filter: recallFilter,
        });
        
        currentSceneId = selectedSceneId;
        showRecallOptions = false;
    }
    
    function saveCurrentAsScene() {
        showNewDialog = true;
    }
    
    function updateScene() {
        if (!selectedSceneId) return;
        
        dispatch('update', { sceneId: selectedSceneId });
    }
    
    function deleteScene() {
        if (!selectedSceneId) return;
        
        if (confirm('Szene wirklich löschen?')) {
            dispatch('delete', { sceneId: selectedSceneId });
            scenes = scenes.filter(s => s.id !== selectedSceneId);
            selectedSceneId = null;
        }
    }
    
    function createNewScene() {
        if (!newSceneName.trim()) return;
        
        dispatch('create', {
            name: newSceneName,
            description: newSceneDescription,
            category: newSceneCategory,
            color: newSceneColor,
        });
        
        // Dialog schließen und zurücksetzen
        showNewDialog = false;
        newSceneName = '';
        newSceneDescription = '';
        newSceneCategory = '';
        newSceneColor = '#27ae60';
    }
    
    function formatDate(dateStr: string): string {
        const date = new Date(dateStr);
        return date.toLocaleDateString('de-DE', {
            day: '2-digit',
            month: '2-digit',
            year: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
        });
    }
    
    function getSelectedScene(): Scene | undefined {
        return scenes.find(s => s.id === selectedSceneId);
    }
</script>

<div class="h-full flex bg-mixer-bg text-white">
    <!-- Szenen-Liste (links) -->
    <div class="w-80 border-r border-gray-700 flex flex-col">
        <!-- Header -->
        <div class="p-4 border-b border-gray-700">
            <h2 class="text-xl font-bold mb-3">🎬 Szenen</h2>
            
            <button
                on:click={saveCurrentAsScene}
                class="w-full py-2 px-4 bg-mixer-accent hover:bg-blue-600 rounded-lg font-medium transition"
            >
                + Aktuelle Einstellungen speichern
            </button>
        </div>
        
        <!-- Szenen-Liste -->
        <div class="flex-1 overflow-y-auto p-2">
            <!-- Kategorien -->
            {#each categories as category}
                <div class="mb-4">
                    <h3 class="text-sm font-semibold text-gray-400 uppercase px-2 mb-1">{category}</h3>
                    
                    {#each groupedScenes[category] as scene}
                        <button
                            on:click={() => selectScene(scene.id)}
                            class="w-full text-left p-3 rounded-lg mb-1 transition
                                   {selectedSceneId === scene.id ? 'bg-mixer-accent' : 'bg-mixer-surface hover:bg-gray-700'}
                                   {currentSceneId === scene.id ? 'ring-2 ring-green-500' : ''}"
                        >
                            <div class="flex items-center gap-2">
                                {#if scene.color}
                                    <div
                                        class="w-3 h-3 rounded-full"
                                        style="background-color: {scene.color}"
                                    />
                                {/if}
                                <span class="font-medium">{scene.name}</span>
                                {#if currentSceneId === scene.id}
                                    <span class="text-xs bg-green-600 px-1 rounded">AKTIV</span>
                                {/if}
                            </div>
                            {#if scene.description}
                                <p class="text-xs text-gray-400 mt-1 truncate">{scene.description}</p>
                            {/if}
                        </button>
                    {/each}
                </div>
            {/each}
            
            <!-- Unkategorisierte Szenen -->
            {#if uncategorizedScenes.length > 0}
                <div class="mb-4">
                    <h3 class="text-sm font-semibold text-gray-400 uppercase px-2 mb-1">Sonstige</h3>
                    
                    {#each uncategorizedScenes as scene}
                        <button
                            on:click={() => selectScene(scene.id)}
                            class="w-full text-left p-3 rounded-lg mb-1 transition
                                   {selectedSceneId === scene.id ? 'bg-mixer-accent' : 'bg-mixer-surface hover:bg-gray-700'}
                                   {currentSceneId === scene.id ? 'ring-2 ring-green-500' : ''}"
                        >
                            <span class="font-medium">{scene.name}</span>
                            {#if currentSceneId === scene.id}
                                <span class="text-xs bg-green-600 px-1 rounded ml-2">AKTIV</span>
                            {/if}
                        </button>
                    {/each}
                </div>
            {/if}
        </div>
    </div>
    
    <!-- Detail-Ansicht (rechts) -->
    <div class="flex-1 flex flex-col">
        {#if selectedSceneId && getSelectedScene()}
            {@const scene = getSelectedScene()}
            
            <!-- Scene Header -->
            <div class="p-6 border-b border-gray-700">
                <div class="flex items-center gap-4">
                    {#if scene?.color}
                        <div
                            class="w-12 h-12 rounded-lg"
                            style="background-color: {scene.color}"
                        />
                    {:else}
                        <div class="w-12 h-12 rounded-lg bg-gray-600 flex items-center justify-center text-2xl">
                            🎬
                        </div>
                    {/if}
                    
                    <div class="flex-1">
                        <h2 class="text-2xl font-bold">{scene?.name}</h2>
                        {#if scene?.description}
                            <p class="text-gray-400">{scene.description}</p>
                        {/if}
                    </div>
                    
                    {#if currentSceneId === selectedSceneId}
                        <span class="px-3 py-1 bg-green-600 rounded-full text-sm font-medium">
                            ✓ Aktiv
                        </span>
                    {/if}
                </div>
                
                <div class="flex gap-2 mt-4 text-sm text-gray-400">
                    <span>Erstellt: {formatDate(scene?.created_at ?? '')}</span>
                    <span>•</span>
                    <span>Geändert: {formatDate(scene?.modified_at ?? '')}</span>
                    {#if scene?.author}
                        <span>•</span>
                        <span>Autor: {scene.author}</span>
                    {/if}
                </div>
            </div>
            
            <!-- Aktionen -->
            <div class="p-6 flex-1">
                <div class="grid grid-cols-2 gap-4 max-w-xl">
                    <!-- Recall Button -->
                    <button
                        on:click={() => showRecallOptions = !showRecallOptions}
                        class="py-4 px-6 bg-green-600 hover:bg-green-500 rounded-xl text-lg font-bold transition"
                    >
                        ▶ RECALL
                    </button>
                    
                    <!-- Update Button -->
                    <button
                        on:click={updateScene}
                        class="py-4 px-6 bg-yellow-600 hover:bg-yellow-500 rounded-xl text-lg font-bold transition"
                    >
                        💾 UPDATE
                    </button>
                </div>
                
                <!-- Recall Optionen -->
                {#if showRecallOptions}
                    <div class="mt-6 p-4 bg-mixer-surface rounded-xl max-w-xl">
                        <h3 class="font-bold mb-3">Recall-Optionen</h3>
                        
                        <div class="grid grid-cols-2 gap-3">
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="checkbox" bind:checked={recallFilter.faders} class="w-5 h-5" />
                                <span>Fader</span>
                            </label>
                            
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="checkbox" bind:checked={recallFilter.mutes} class="w-5 h-5" />
                                <span>Mutes</span>
                            </label>
                            
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="checkbox" bind:checked={recallFilter.solos} class="w-5 h-5" />
                                <span>Solos</span>
                            </label>
                            
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="checkbox" bind:checked={recallFilter.pans} class="w-5 h-5" />
                                <span>Pan</span>
                            </label>
                            
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="checkbox" bind:checked={recallFilter.eq} class="w-5 h-5" />
                                <span>EQ</span>
                            </label>
                            
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="checkbox" bind:checked={recallFilter.routing} class="w-5 h-5" />
                                <span>Routing</span>
                            </label>
                            
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="checkbox" bind:checked={recallFilter.names} class="w-5 h-5" />
                                <span>Namen</span>
                            </label>
                        </div>
                        
                        <button
                            on:click={recallScene}
                            class="mt-4 w-full py-3 bg-green-600 hover:bg-green-500 rounded-lg font-bold"
                        >
                            Szene laden
                        </button>
                    </div>
                {/if}
                
                <!-- Danger Zone -->
                <div class="mt-8 pt-6 border-t border-gray-700 max-w-xl">
                    <h3 class="text-red-400 font-bold mb-3">Gefahrenzone</h3>
                    <button
                        on:click={deleteScene}
                        class="py-2 px-4 bg-red-900 hover:bg-red-700 rounded-lg text-red-200"
                    >
                        🗑️ Szene löschen
                    </button>
                </div>
            </div>
        {:else}
            <!-- Keine Szene ausgewählt -->
            <div class="flex-1 flex items-center justify-center text-gray-500">
                <div class="text-center">
                    <div class="text-6xl mb-4">🎬</div>
                    <p class="text-xl">Wähle eine Szene aus</p>
                    <p class="text-sm mt-2">oder speichere die aktuellen Einstellungen als neue Szene</p>
                </div>
            </div>
        {/if}
    </div>
    
    <!-- Neue Szene Dialog -->
    {#if showNewDialog}
        <div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
            <div class="bg-mixer-surface rounded-2xl p-6 w-full max-w-md">
                <h2 class="text-xl font-bold mb-4">Neue Szene erstellen</h2>
                
                <div class="space-y-4">
                    <div>
                        <label class="block text-sm text-gray-400 mb-1">Name *</label>
                        <input
                            type="text"
                            bind:value={newSceneName}
                            placeholder="Szenen-Name"
                            class="w-full px-4 py-2 bg-mixer-bg rounded-lg border border-gray-600 focus:border-mixer-accent focus:outline-none"
                        />
                    </div>
                    
                    <div>
                        <label class="block text-sm text-gray-400 mb-1">Beschreibung</label>
                        <textarea
                            bind:value={newSceneDescription}
                            placeholder="Optionale Beschreibung"
                            rows="2"
                            class="w-full px-4 py-2 bg-mixer-bg rounded-lg border border-gray-600 focus:border-mixer-accent focus:outline-none resize-none"
                        />
                    </div>
                    
                    <div>
                        <label class="block text-sm text-gray-400 mb-1">Kategorie</label>
                        <input
                            type="text"
                            bind:value={newSceneCategory}
                            placeholder="z.B. Live, Produktion, Vorlagen"
                            list="categories"
                            class="w-full px-4 py-2 bg-mixer-bg rounded-lg border border-gray-600 focus:border-mixer-accent focus:outline-none"
                        />
                        <datalist id="categories">
                            {#each categories as cat}
                                <option value={cat} />
                            {/each}
                        </datalist>
                    </div>
                    
                    <div>
                        <label class="block text-sm text-gray-400 mb-1">Farbe</label>
                        <div class="flex gap-2">
                            {#each ['#27ae60', '#3498db', '#9b59b6', '#e74c3c', '#f39c12', '#1abc9c'] as color}
                                <button
                                    on:click={() => newSceneColor = color}
                                    class="w-8 h-8 rounded-lg transition
                                           {newSceneColor === color ? 'ring-2 ring-white ring-offset-2 ring-offset-mixer-surface' : ''}"
                                    style="background-color: {color}"
                                />
                            {/each}
                        </div>
                    </div>
                </div>
                
                <div class="flex gap-3 mt-6">
                    <button
                        on:click={() => showNewDialog = false}
                        class="flex-1 py-2 px-4 bg-gray-700 hover:bg-gray-600 rounded-lg"
                    >
                        Abbrechen
                    </button>
                    <button
                        on:click={createNewScene}
                        disabled={!newSceneName.trim()}
                        class="flex-1 py-2 px-4 bg-mixer-accent hover:bg-blue-600 rounded-lg font-medium
                               disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        Speichern
                    </button>
                </div>
            </div>
        </div>
    {/if}
</div>
