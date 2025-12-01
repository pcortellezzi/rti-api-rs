# Plan de Refonte du Connecteur Rithmic-RS

L'objectif est de créer un connecteur robuste, thread-safe et asynchrone pour l'API Rithmic, en remplaçant l'implémentation défaillante basée sur `kameo` par une architecture native Tokio.

## Phase 1 : Nettoyage et Fondations

- [x] **Nettoyage des dépendances** : Retirer `kameo` et nettoyer `Cargo.toml` pour ne garder que l'essentiel (`tokio`, `prost`, `tokio-tungstenite`, `tracing`).
- [x] **Architecture de base** : Créer une structure `Client` vide et un point d'entrée propre dans `lib.rs`.

## Phase 2 : Couche de Transport et Protocole

- [x] **Gestion de la Connexion (WebSocket)** : Implémenter la connexion SSL/TLS vers les serveurs Rithmic via `tokio-tungstenite`.
- [x] **Sérialisation/Désérialisation** : Créer les helpers pour transformer les messages Protobuf en bytes et vice-versa (`api/decoder.rs`).
- [x] **Boucle d'Événements (Event Loop)** : Implémenter la boucle principale qui écoute le socket (`plants/worker.rs`).

## Phase 3 : Gestion de Session et Requêtes

- [x] **Login & Authentification** : Implémenter le flux de connexion (`request_login`, `response_login`) dans le Worker.
- [ ] **Heartbeat** : Implémenter le mécanisme de ping/pong automatique (Prévu mais pas encore câblé dans le client).
- [x] **Système Request/Response** : Implémenter un système de corrélation (Map `request_id` -> `oneshot::Sender`).

## Phase 4 : Implémentation Fonctionnelle (Client Unifié)

- [x] **Architecture Client Unifié** : `RithmicClient` gère les sous-systèmes (Plants).
- [x] **Market Data** : Implémentation de la connexion au Ticker Plant et abonnement.
- [ ] **Order Management** : À implémenter (similaire au Ticker Plant mais avec Order Plant).
- [ ] **Reference Data** : À implémenter.

## Phase 5 : Consolidation et Exemple

- [x] **Exemple complet** : `examples/simple_market_data.rs`.