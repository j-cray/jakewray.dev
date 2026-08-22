use leptos::prelude::*;

#[component]
pub fn ScheduleModal(
    show: Signal<bool>,
    on_close: Callback<()>,
    scheduled_datetime: Signal<String>,
    set_scheduled_datetime: WriteSignal<String>,
    set_save_status: WriteSignal<String>,
    on_confirm: Callback<(&'static str, Option<String>)>,
) -> impl IntoView {
    move || {
        if show.get() {
            Some(view! {
                <div class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
                    <div class="bg-white rounded-xl shadow-2xl border max-w-md w-full p-6 animate-in fade-in zoom-in duration-200">
                        <h3 class="text-xl font-bold mb-3 text-gray-900 flex items-center gap-2">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-sky-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            "Schedule Publication"
                        </h3>
                        <p class="text-sm text-gray-600 mb-4">
                            "Choose when this post should automatically become visible on your blog and portfolio."
                        </p>

                        <div class="mb-6">
                            <label class="block font-bold mb-2 text-gray-700 text-sm">"Publish Date & Time"</label>
                            <input
                                type="datetime-local"
                                class="w-full p-3 border rounded-lg text-gray-800 font-medium focus:ring-2 focus:ring-sky-500 focus:outline-none"
                                prop:value=scheduled_datetime.get()
                                on:input=move |ev| set_scheduled_datetime.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="flex justify-end gap-3">
                            <button
                                type="button"
                                class="btn btn-secondary"
                                on:click=move |_| on_close.run(())
                            >
                                "Cancel"
                            </button>
                            <button
                                type="button"
                                class="btn btn-primary flex items-center gap-2"
                                on:click=move |_| {
                                    let dt = scheduled_datetime.get();
                                    if dt.trim().is_empty() {
                                        set_save_status.set("Please select a date and time for scheduling.".to_string());
                                        return;
                                    }
                                    on_close.run(());
                                    on_confirm.run(("scheduled", Some(dt)));
                                }
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                </svg>
                                "Confirm Schedule"
                            </button>
                        </div>
                    </div>
                </div>
            })
        } else {
            None
        }
    }
}
