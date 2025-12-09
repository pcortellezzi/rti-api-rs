## Plan de Migration vers `eyre` pour la Gestion des Erreurs et Uniformisation de `tracing`

**Objectif :** Remplacer `anyhow` par `eyre` pour une gestion des erreurs plus riche et s'assurer que `tracing` est utilisé de manière cohérente pour la journalisation dans l'ensemble du projet.

### Étapes de la Migration :

1.  **Mise à Jour de `Cargo.toml`**
    *   Ajouter `eyre = "0.6"` comme dépendance.
    *   Supprimer la dépendance `anyhow`.
    *   Vérifier que `tracing` et `tracing-subscriber` sont présents et configurés.

2.  **Migration du Code de Gestion des Erreurs (`anyhow` vers `eyre`)**

    *   **Recherche et Remplacement Global (avec vérification contextuelle) :**
        *   Remplacer `anyhow::Error` par `eyre::Report`.
        *   Remplacer `Result<(), anyhow::Error>` par `eyre::Result<()>`.
        *   Remplacer `anyhow::anyhow!(...)` par `eyre!(...)`.
        *   Remplacer `.context(...)` et `.wrap_err(...)` par leurs équivalents `eyre` si nécessaire (la syntaxe est souvent similaire).
        *   Mettre à jour les `use anyhow::*` en `use eyre::{eyre, Report, Result}` ou similaire.
        *   Vérifier toutes les fonctions qui retournent `Result` et ajuster les types génériques si elles utilisaient `anyhow::Error`.

    *   **Fichiers Cibles Principaux :**
        *   `src/client.rs`
        *   `src/ws.rs`
        *   `src/plants/worker.rs`
        *   `examples/full_usage.rs`
        *   `examples/simple_market_data.rs`
        *   Les fichiers de tests (e.g., `tests/compliance.rs`, `tests/receiver_api_tests.rs`, `tests/integration_test.rs`)

3.  **Uniformisation de l'Utilisation de `tracing`**

    *   **Vérification de l'Initialisation :** S'assurer que `tracing_subscriber::fmt::init();` est appelé une seule fois au début de `main` ou du point d'entrée principal.
    *   **Utilisation Cohérente des Macros :** Examiner tous les fichiers pour s'assurer que la journalisation utilise systématiquement les macros `tracing::info!`, `tracing::debug!`, `tracing::error!`, `tracing::warn!`, etc., au lieu de `println!` ou `eprintln!` non-formatés.
    *   **Passage des `Result` :** Utiliser `Result.map_err(|e| error!("{:#}", e))` ou `Result.wrap_err_with(...)` pour logger les erreurs de manière structurée avec `tracing`.

4.  **Vérification et Tests**
    *   Compiler le projet (`cargo check`).
    *   Exécuter les tests (`cargo test`).
    *   Lancer les exemples (`cargo run --example full_usage`).

**Considérations :**
*   **Types de Retour :** `eyre::Result` est un alias pour `Result<T, eyre::Report>`, donc les signatures de fonction devront être mises à jour.
*   **Propagation des Erreurs :** S'assurer que l'opérateur `?` propage correctement les `eyre::Report`.
*   **Messages d'Erreur :** `eyre` offre un formatage d'erreur supérieur, notamment avec le trait `Debug` pour `Report`, ce qui sera un avantage pour le débogage.
