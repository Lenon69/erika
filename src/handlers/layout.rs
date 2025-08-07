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
                link rel="preconnect" href="https://cdn.jsdelivr.net";
                script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" {}
                script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.6/dist/htmx.min.js" defer {}
                script src="https://cdn.jsdelivr.net/npm/alpinejs@3.14.9/dist/cdn.min.js" defer {}
                title { (title) " - Erika" }
            }
            body x-data="{ modalOpen: false }" class="bg-gray-900 text-gray-200 antialiased" {

                // --- NOWA SEKCJA: NAGŁÓWEK Z NAWIGACJĄ ---
                header class="bg-gray-800/70 backdrop-blur-lg shadow-md sticky top-0 z-50" {
                    nav class="container mx-auto px-4 py-3 flex justify-between items-center" {
                        // Link do strony głównej (logo)
                        a href="/" class="text-2xl font-bold text-white hover:text-purple-400 transition-colors" {
                            "Erika"
                        }

                        // Linki do logowania i rejestracji
                        div class="flex items-center gap-4" {
                            a href="/login" class="text-gray-300 hover:text-white transition-colors" { "Zaloguj się" }
                            a href="/register" class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" {
                                "Zarejestruj się"
                            }
                        }
                    }
                }

                // Używamy `main` dla lepszej semantyki HTML
                main class="container mx-auto mt-10 px-4" {
                    // Renderujemy unikalną zawartość
                    (content)
                }


           // --- KOMPONENT MODALA ---
            // Jest on zawsze na stronie, ale domyślnie ukryty (`x-show="modalOpen"`)
            // Zmieniamy `@htmx:after-request.window` na `@close-modal.window`
            div x-show="modalOpen"
                "@close-modal.window"="modalOpen = false" // Zamknij modal tylko po otrzymaniu sygnału
                "x-on:keydown.escape.window"="modalOpen = false"
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/70"
                style="display: none;" {
                    // Półprzezroczyste tło, które zamyka modal po kliknięciu
                    div class="absolute inset-0" "@click"="modalOpen = false" {}

                    // Kontener na treść, którą dynamicznie załaduje HTMX
                    div id="modal-content"
                         x-show="modalOpen"
                         x-transition // Prosta animacja pojawiania się
                         class="relative bg-gray-800 rounded-lg shadow-xl p-8 z-10" {}
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
