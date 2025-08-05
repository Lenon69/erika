// src/handlers/layout.rs

use maud::{DOCTYPE, Markup, html};

// Ta funkcja będzie naszym głównym szablonem strony.
// Przyjmuje tytuł strony i jej unikalną zawartość (content).
pub fn page(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="pl" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";

                // Tutaj dodajemy skrypt Tailwind CSS v4 z CDN
                script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" {}

                title { (title) " - LenonDev" }
            }
            // Dodajemy klasy Tailwind do body, aby ustawić ciemny motyw
            body class="bg-gray-900 text-gray-200 antialiased" {
                // Kontener centrujący zawartość i dodający marginesy
                div class="container mx-auto mt-10 px-4" {
                    // Renderujemy tutaj unikalną zawartość przekazaną do funkcji
                    (content)
                }
            }
        }
    }
}

// Nowa funkcja do wyświetlania prostych stron z komunikatem
pub fn info_page(title: &str, message: &str, link: Option<(&str, &str)>) -> Markup {
    let content = html! {
        div class="max-w-xl mx-auto bg-gray-800 p-8 rounded-lg shadow-lg text-center" {
            h1 class="text-3xl font-bold text-white mb-6" { (message) }
            @if let Some((link_href, link_text)) = link {
                a href=(link_href) class="inline-block bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" {
                    (link_text)
                }
            }
        }
    };
    page(title, content)
}
