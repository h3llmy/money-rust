# Hexagonal Finance API: Frontend Integration Blueprint

This document provides a strictly structured reference for AI assistants and developers to build the frontend.

## 📡 Connection Details
- **Base URL**: `http://localhost:5008/api/v1`
- **Content-Type**: `application/json`

---

## 🏗️ Data Models (Schemas)

### Core Entities
```typescript
type UUID = string; // Standard UUID v4
type Decimal = number | string; // Use string for high precision in JS

interface Pocket {
  id: UUID;
  name: string;
  pocket_type: "bank" | "cash" | "ewallet" | "other";
  currency: string;
  balance: Decimal;
}

interface Category {
  id: UUID;
  name: string;
  type: "income" | "expense";
}

interface Transaction {
  id: UUID;
  pocket_id: UUID;
  category_id?: UUID;
  amount: Decimal;
  type: "income" | "expense" | "transfer";
  title: string;
  transaction_time: string; // ISO 8601
  destination_pocket_id?: UUID; // Required only for "transfer"
}

interface Notification {
  id: UUID;
  app_package: string;
  raw_title?: string;
  raw_body: string;
  received_at: string; // ISO 8601
  status: "pending" | "processed" | "failed" | "ignored";
  transaction_id?: UUID;
}
```

### Standard Response Wrappers
```typescript
interface ApiResponse<T> {
  data: T;
}

interface PaginationResponse<T> {
  data: T[];
  page: number;
  limit: number;
  total_data: number;
  total_page: number;
}
```

---

## 🛣️ API Endpoints

### 1. Pockets
| Endpoint | Method | Body | Response |
| :--- | :--- | :--- | :--- |
| `/pockets` | `GET` | - | `PaginationResponse<Pocket>` |
| `/pockets` | `POST` | `CreatePocketRequest` | `ApiResponse<Pocket>` |
| `/pockets/:id` | `GET` | - | `ApiResponse<Pocket>` |
| `/pockets/:id` | `PUT` | `UpdatePocketRequest` | `ApiResponse<Pocket>` |
| `/pockets/:id` | `DELETE` | - | `204 No Content` |

**Pagination Query Params**: `page`, `limit`, `search`.

---

### 2. Transactions
| Endpoint | Method | Body | Response |
| :--- | :--- | :--- | :--- |
| `/transactions` | `GET` | - | `PaginationResponse<Transaction>` |
| `/transactions` | `POST` | `CreateTransactionRequest` | `ApiResponse<Transaction>` |
| `/transactions/:id` | `DELETE` | - | `204 No Content` |

**Filters & Sorting**:
- `pocket_id`: Filter by source.
- `search`: Partial title match.
- `sort`: `amount` | `transaction_time`.
- `sort_order`: `Asc` | `Desc`.

---

### 3. Categories
| Endpoint | Method | Body | Response |
| :--- | :--- | :--- | :--- |
| `/categories` | `GET` | - | `PaginationResponse<Category>` |
| `/categories` | `POST` | `CreateCategoryRequest` | `ApiResponse<Category>` |

---

### 4. Automation Inbox
| Endpoint | Method | Body | Response |
| :--- | :--- | :--- | :--- |
| `/inbox/sync` | `POST` | `CreateNotificationRequest[]` | `202 Accepted` |
| `/inbox/unresolved` | `GET` | - | `PaginationResponse<Notification>` |
| `/inbox/:id/resolve` | `POST` | `ResolveNotificationRequest` | `ApiResponse<string>` |
| `/inbox/:id/ignore` | `DELETE` | - | `204 No Content` |

---

## 🔄 Business Logic & Workflows

### The Notification Lifecycle
1.  **Ingestion**: Frontend sends raw notifications via `/inbox/sync`.
2.  **Processing (Backend)**: AI (Ollama) attempts to parse. 
    - Success → Transaction created, Notification `status` = `processed`.
    - Fail → Notification `status` = `failed`.
3.  **Resolution (Frontend)**: User views `failed` notifications in `/inbox/unresolved` and manually assigns a Pocket/Category via `/resolve`.

### Transaction Rules
- **Income**: Increases `pocket.balance`.
- **Expense**: Decreases `pocket.balance`.
- **Transfer**: Decreases `source_pocket.balance` and increases `destination_pocket.balance`.

---

## 🤖 AI Prompting Tips
When asking an AI to build a screen for this API, copy-paste the **Data Models** and **API Endpoints** sections. 

**Example Prompt**:
> "Using the provided Finance API blueprint, create a React component for the 'Unresolved Notifications' list. It should use TanStack Query to fetch from `/api/v1/inbox/unresolved` and provide a form to resolve each notification using `/api/v1/inbox/:id/resolve`."
