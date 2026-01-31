use leptos::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="container py-12 max-w-2xl bg-white p-8 rounded-lg shadow-sm border border-gray-100">
            <h1 class="text-4xl mb-6 font-bold text-gray-900 border-b border-gray-100 pb-4">"About Me"</h1>
            
            <div class="prose prose-lg text-gray-700 leading-relaxed">
                <p class="mb-6">
                    "I am a journalist, developer, and photographer based in Northern British Columbia. I have a passion for uncovering stories that matter and documenting the world around me through both words and images."
                </p>

                <p class="mb-6">
                    "Currently, I am expanding my horizons into software development, building tools and applications that bridge the gap between storytelling and technology. This website itself is a testament to that journey—a work in progress where I explore new ideas and showcase my evolving portfolio."
                </p>

                <h3 class="text-2xl font-semibold mt-8 mb-4 text-gray-800">"Journalism"</h3>
                <p class="mb-4">
                    "My reporting focuses on community issues, Indigenous culture, and public interest stories in the Terrace and Kitimat regions. I believe in the power of local journalism to inform communities and hold power to account."
                </p>

                 <h3 class="text-2xl font-semibold mt-8 mb-4 text-gray-800">"Development"</h3>
                <p class="mb-4">
                    "As a developer, I am interested in Rust, web technologies, and building efficient, user-focused applications. I am currently working on several projects that integrate my diverse interests."
                </p>
            </div>
        </div>
    }
}
