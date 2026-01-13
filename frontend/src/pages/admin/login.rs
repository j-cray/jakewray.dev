use leptos::logging;
use leptos::prelude::*;
use leptos_router::hooks::*;

#[component]
pub fn AdminLoginPage() -> impl IntoView {
    let (username, set_username) = signal("".to_string());
    let (_password, set_password) = signal("".to_string());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        // Todo: Call server action to login
        logging::log!("Login attempt: {}", username.get());
        let navigate = use_navigate();
        navigate("/admin/dashboard", Default::default());
    };

    view! {
        <div class="flex items-center justify-center min-h-screen bg-gray-100">
            <div class="card w-full max-w-md bg-white p-8 rounded-lg shadow-md">
                <h1 class="text-2xl font-bold mb-6 text-center">"Admin Login"</h1>
                <form on:submit=on_submit class="flex flex-col gap-4">
                    <input
                        type="text"
                        placeholder="Username"
                        class="p-3 border rounded-md"
                        on:input=move |ev| set_username.set(event_target_value(&ev))
                    />
                    <input
                        type="password"
                        placeholder="Password"
                        class="p-3 border rounded-md"
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                    />
                    <button type="submit" class="bg-black text-white p-3 rounded-md font-bold hover:bg-gray-800 transition">
                        "Login"
                    </button>
                </form>
            </div>
        </div>
    }
}
