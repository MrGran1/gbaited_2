# lol-pronos-api

REST API Rust (Axum + SQLx) pour l'application mobile de pronostics LoL Esport.

## Prérequis

- Rust 1.75+ (`rustup update stable`)
- MySQL 8.0 compatible (MariaDB 10.5+ fonctionne aussi)
- La base de données `gbaited` créée à partir du dump `gbaited_dump.sql`

## Configuration

Copier `.env.example` en `.env` et remplir les valeurs :

```
DATABASE_URL=mysql://user:password@localhost:3306/gbaited
JWT_SECRET=une_cle_secrete_dau_moins_32_caracteres
JWT_EXPIRY_HOURS=24
PORT=3000
RUST_LOG=info
```

## Lancer l'API

```bash
cp .env.example .env
# éditer .env avec vos vraies valeurs
cargo run
```

## Endpoints

### Auth (public)
| Méthode | Route | Description |
|---------|-------|-------------|
| POST | `/api/auth/register` | Créer un compte |
| POST | `/api/auth/login` | Se connecter (retourne un JWT) |

### Compétitions (public)
| Méthode | Route | Description |
|---------|-------|-------------|
| GET | `/api/competitions` | Liste des compétitions |
| GET | `/api/competitions/:id` | Détail d'une compétition |
| GET | `/api/competitions/:id/matches` | Matchs d'une compétition |

### Matchs (public / protégé)
| Méthode | Route | Description |
|---------|-------|-------------|
| GET | `/api/matches` | Tous les matchs |
| GET | `/api/matches/:id` | Détail d'un match |
| PUT | `/api/matches/:id` | Mettre à jour score/status/winner 🔒 |

### Équipes (public)
| Méthode | Route | Description |
|---------|-------|-------------|
| GET | `/api/teams` | Liste des équipes |
| GET | `/api/teams/:id` | Détail d'une équipe |

### Tournois (🔒 = authentifié)
| Méthode | Route | Description |
|---------|-------|-------------|
| GET | `/api/tournaments` | Liste des tournois |
| POST | `/api/tournaments` | Créer un tournoi 🔒 |
| GET | `/api/tournaments/:id` | Détail d'un tournoi |
| POST | `/api/tournaments/:id/join` | Rejoindre un tournoi 🔒 |
| GET | `/api/tournaments/:id/members` | Membres d'un tournoi |
| GET | `/api/tournaments/:id/leaderboard` | Classement du tournoi |

### Paris (🔒 authentifié requis)
| Méthode | Route | Description |
|---------|-------|-------------|
| GET | `/api/paris` | Mes paris |
| POST | `/api/paris` | Placer un pari |
| PUT | `/api/paris/:id` | Modifier un pari (match non terminé) |
| GET | `/api/paris/match/:match_id` | Paris sur un match |

### Utilisateur (🔒 authentifié requis)
| Méthode | Route | Description |
|---------|-------|-------------|
| GET | `/api/users/me` | Mon profil |
| PATCH | `/api/users/me` | Modifier pseudo / mot de passe |

## Format JWT

Ajouter dans les headers des requêtes protégées :
```
Authorization: Bearer <token>
```

## Format des réponses d'erreur

```json
{ "error": "message d'erreur" }
```

Codes HTTP : 400 Bad Request · 401 Unauthorized · 404 Not Found · 409 Conflict · 500 Internal Server Error
