use leptos::prelude::*;

#[component]
pub fn AdminSyncManager() -> impl IntoView {
    let (sync_status, set_sync_status) = signal("Idle".to_string());
    let (last_sync, set_last_sync) = signal("Never".to_string());

    let trigger_sync = move |_| {
        set_sync_status.set("Syncing...".to_string());
        // Server Action TODO

        // Mock completion
        set_timeout(
            move || {
                set_sync_status.set("Idle".to_string());
                set_last_sync.set("Just now".to_string());
            },
            std::time::Duration::from_secs(2),
        );
    };

    view! {
        <div class="container py-12 max-w-2xl">
            <h1 class="text-3xl mb-8">"Sync Manager"</h1>

            <div class="card mb-8">
                <div class="flex justify-between items-center mb-4">
                    <h3 class="text-xl font-bold">"Terrace Standard"</h3>
                    <span class="px-3 py-1 bg-green-100 text-green-800 rounded-full text-sm">"Active"</span>
                </div>
                <p class="mb-2">"Mirroring articles by: "<span class="font-mono bg-gray-100 px-2 py-1 rounded">"Jake Wray"</span></p>
                <div class="flex justify-between items-center mt-6">
                    <div>
                        <p class="text-sm text-muted">"Status: " <span class="font-medium text-black">{sync_status}</span></p>
                        <p class="text-sm text-muted">"Last Sync: " <span class="font-medium text-black">{last_sync}</span></p>
                    </div>
                    <button
                        class="px-6 py-2 bg-black text-white rounded-md font-bold disabled:opacity-50"
                        on:click=trigger_sync
                        disabled=move || sync_status.get() == "Syncing..."
                    >
                        {move || if sync_status.get() == "Syncing..." { "Syncing..." } else { "Sync Now" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
