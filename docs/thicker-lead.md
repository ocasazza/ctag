# Thicker Lead

This document outlines the recommended approach for building a custom Jira Service Management (JSM) portal using Vue.js, leveraging Okta as your federated Identity Provider (IdP) and Atlassian's OAuth 2.0 (3LO) for API authentication. This guide assumes that all users of the custom portal have existing Atlassian licenses and are managed via Okta, with Okta groups used for assigning permissions within Atlassian.

## Recommended Architecture and Workflow

Given that everyone has a license and your organization uses Okta for identity management with groups pushed for permissions, the most robust and standard approach involves:

1.  **User Identity & Permissions:**
    * Users are full Atlassian accounts.
    * Okta serves as your federated IdP, integrated with your Atlassian organization via Atlassian Access.
    * User provisioning (creation, updates, deactivation) and group memberships are managed in Okta and synced to your Atlassian organization (likely via SCIM).
    * Within Jira Service Management, these synchronized groups (or Atlassian groups mapped from them) define project roles, issue security levels, queue access, knowledge base permissions, etc.

2.  **Custom Vue.js Portal Authentication & Authorization (OAuth 2.0 3LO):**
    * Your custom portal will utilize Atlassian's OAuth 2.0 (3-legged OAuth) authorization code grant flow. The authentication step within this flow will seamlessly route through Okta due to the federation.

This setup allows your custom portal to securely interact with JSM APIs on behalf of authenticated, licensed users, respecting their existing permissions.

## High-Level System Architecture

This diagram provides an overview of the main components and their interactions:

```mermaid
graph TD
    User[<actor> User] -- Interacts --> VueFE[Vue.js Frontend]

    subgraph Custom Vue.js Portal
        VueFE
        PortalBE[Custom Portal Backend]
    end

    VueFE -- Login Request --> PortalBE
    PortalBE -- OAuth 2.0 (3LO) Flow --> AtlassianAuth[Atlassian Auth Server]
    AtlassianAuth -- Federation --> Okta[Okta IdP]
    Okta -- Authenticates --> User
    AtlassianAuth -- Tokens --> PortalBE

    PortalBE -- JSM API Calls (with Access Token) --> JSM_APIs[Jira Service Management APIs]

    subgraph Atlassian Cloud
        AtlassianAccess[Atlassian Access]
        JSM_APIs
    end

    Okta -- SCIM Provisioning (Users/Groups) --> AtlassianAccess

    style User fill:#A6E22E,stroke:#333,stroke-width:2px
    style VueFE fill:#66D9EF,stroke:#333,stroke-width:2px
    style PortalBE fill:#66D9EF,stroke:#333,stroke-width:2px
    style Okta fill:#F92672,stroke:#333,stroke-width:2px
    style AtlassianAuth fill:#FD971F,stroke:#333,stroke-width:2px
    style AtlassianAccess fill:#FD971F,stroke:#333,stroke-width:2px
    style JSM_APIs fill:#AE81FF,stroke:#333,stroke-width:2px
````

**Explanation of the Architecture Diagram:**

  * The **User** interacts with the **Vue.js Frontend**.
  * Authentication is orchestrated by the **Custom Portal Backend**, involving redirection through the **Atlassian Auth Server** to **Okta** (due to your Atlassian Access federation).
  * The **Custom Portal Backend** obtains an `access_token` and uses it to make calls to the **Jira Service Management APIs**.
  * Separately (usually an ongoing admin process), **Okta** handles user and group provisioning into your Atlassian organization via **Atlassian Access** using SCIM. These provisioned users and groups are then used within JSM for permissions.

## OAuth 2.0 (3LO) Authentication Flow

This is the detailed process for authenticating users and obtaining API access tokens:

1.  **App Registration:**

      * Register your custom Vue.js portal as an OAuth 2.0 (3LO) application in your Atlassian developer console (developer.atlassian.com).
      * Specify:
          * **Redirect URIs:** The URI(s) in your Vue.js application (handled by your backend) where users will be redirected after authorizing your app.
          * **Scopes:** The permissions your custom portal needs (e.g., `read:jira-work`, `write:jira-work`, `read:servicedesk-request`, `write:servicedesk-request`, `read:jira-user`). Choose minimally.

2.  **Authentication Flow Steps:**

    1.  **Initiate Login:** User clicks "Login" in the Vue.js portal.
    2.  **Redirect to Atlassian:** Your application redirects the user to the Atlassian authorization server endpoint (with `client_id`, `redirect_uri`, `scope`, `response_type=code`, and a `state` parameter).
    3.  **Authentication via Okta:** Atlassian redirects the user to your Okta instance for authentication. The user signs in with their Okta credentials.
    4.  **Grant Consent:** After Okta authentication, the user is redirected to Atlassian to authorize your custom portal (if consent is not already granted).
    5.  **Authorization Code:** Atlassian redirects the user to your specified `redirect_uri` with an `authorization_code`.
    6.  **Token Exchange (Backend Operation):** Your Custom Portal Backend securely exchanges the `authorization_code` (along with your app's `client_id` and `client_secret`) for an `access_token` and a `refresh_token` from the Atlassian token endpoint. **This step must occur on the server-side.**
    7.  **Store Tokens Securely:** The backend stores the `refresh_token` securely (e.g., encrypted in a database) and manages the `access_token` for API calls, potentially establishing a session for the frontend.

3.  **Making JSM API Calls:**

      * Your custom portal (via its backend) uses the `access_token` in the `Authorization: Bearer <access_token>` header for JSM REST API requests.
      * API requests are executed with the authenticated user's permissions, as enforced by JSM based on their group memberships (synced from Okta).

<!-- end list -->

```mermaid
sequenceDiagram
    actor User
    participant VueApp as Vue.js Frontend (Browser)
    participant PortalBE as Custom Portal Backend
    participant AtlasAuth as Atlassian Authorization Server
    participant OktaIdP as Okta (Identity Provider)

    User->>VueApp: 1. Clicks Login
    VueApp->>User: 2. Redirect to Atlassian Auth Server (params: client_id, redirect_uri, scope, response_type=code, state)
    User->>AtlasAuth: 3. Navigates to Atlassian Auth Server
    AtlasAuth->>User: 4. Redirect to Okta (due to federation)
    User->>OktaIdP: 5. Authenticates with Okta credentials
    OktaIdP-->>User: 6. Authentication success, redirect back to Atlassian
    User->>AtlasAuth: 7. Returns to Atlassian Auth Server with Okta auth proof
    AtlasAuth->>User: 8. Prompt for consent (to allow Portal to access JSM data)
    User->>AtlasAuth: 9. Grants consent
    AtlasAuth->>User: 10. Redirect to Portal's redirect_uri (VueApp/PortalBE) with Authorization Code & state
    User->>VueApp: 11. Browser navigates to redirect_uri
    VueApp->>PortalBE: 12. Send Authorization Code & state to Backend

    Note over PortalBE, AtlasAuth: Token Exchange (Server-to-Server)
    PortalBE->>AtlasAuth: 13. Exchange Auth Code for Tokens (params: client_id, client_secret, code, grant_type, redirect_uri)
    AtlasAuth-->>PortalBE: 14. Returns Access Token & Refresh Token

    PortalBE->>PortalBE: 15. Securely store Refresh Token (e.g., DB)
    PortalBE->>VueApp: 16. Establish session, provide Access Token (e.g., HttpOnly cookie or secure delivery)
    VueApp->>PortalBE: 17. Request JSM data (proxied or direct)
    PortalBE->>JSM_API as JSM Cloud APIs: 18. Call JSM API with Access Token
    JSM_API-->>PortalBE: 19. Return JSM data
    PortalBE-->>VueApp: 20. Return data to Frontend
    VueApp->>User: 21. Display JSM data
```

**Explanation of the Authentication Flow Diagram:**

  * **Steps 1-7:** User initiates login, is redirected via Atlassian to Okta for credential verification due to federation.
  * **Steps 8-9:** User grants your custom app permission to access their Atlassian/JSM data (OAuth consent).
  * **Steps 10-12:** Atlassian sends an `authorization_code` back to your application's `redirect_uri`. The frontend passes this to your backend.
  * **Steps 13-14 (Backend Operations):** Your **Custom Portal Backend** securely exchanges the `authorization_code` for an `access_token` and `refresh_token` directly with Atlassian.
  * **Steps 15-16:** Backend stores the `refresh_token` securely and manages the session. The `access_token` is used for API calls.
  * **Steps 17-21:** The `access_token` is used for authenticated requests to JSM Cloud APIs, operating in the user's context.

## User Provisioning & Permissions Flow

This diagram illustrates how Okta groups are used to manage JSM permissions, which are then respected by API calls.

```mermaid
flowchart LR
    subgraph Okta Environment
        OktaAdmin[Okta Admin] -- Manages --> OktaUsers[Okta Users]
        OktaUsers -- Assigned to --> OktaPushGroups[Okta Push Groups]
    end

    OktaPushGroups -- SCIM Sync --> AtlassianOrg[Atlassian Organization (Users & Groups via Atlassian Access)]

    subgraph Atlassian & JSM Configuration
        AtlassianAdmin[Atlassian/JSM Admin] -- Configures --> JSMPermissions[JSM Project Roles, Customer Permissions, etc.]
        AtlassianOrg -- Used in --> JSMPermissions
    end

    CustomPortal[Custom Vue.js Portal] -- API Call with User's Access Token --> JSM_API[JSM API]
    JSM_API -- Enforces Permissions based on --> JSMPermissions

    style OktaAdmin fill:#F92672,stroke:#333
    style OktaUsers fill:#F92672,stroke:#333
    style OktaPushGroups fill:#F92672,stroke:#333
    style AtlassianOrg fill:#FD971F,stroke:#333
    style AtlassianAdmin fill:#FD971F,stroke:#333
    style JSMPermissions fill:#AE81FF,stroke:#333
    style CustomPortal fill:#66D9EF,stroke:#333
    style JSM_API fill:#AE81FF,stroke:#333
```

**Explanation of Provisioning & Permissions Diagram:**

  * **Okta Admins** manage users and assign them to **Okta Push Groups**.
  * These users and groups are synced via SCIM to your **Atlassian Organization** (managed by Atlassian Access).
  * **Atlassian/JSM Admins** use these synced groups to configure **JSM Permissions** (project roles, access levels, etc.).
  * When an authenticated user (via your **Custom Vue.js Portal**) makes an API call, the **JSM API** enforces permissions based on the user's group memberships and their associated JSM roles.

## Advantages of this Approach

  * **Standard and Secure:** Leverages Atlassian's standard OAuth 2.0 (3LO) mechanism.
  * **User-Contextual Permissions:** API calls automatically operate within the logged-in user's permissions as defined in JSM (driven by your Okta groups).
  * **Leverages Existing Identity Infrastructure:** Seamlessly integrates with your Okta and Atlassian Access setup.
  * **Simplified Licensing Model:** Assumes all users are licensed Atlassian accounts, avoiding complexities of API access for non-licensed portal-only customers.

## Key JavaScript/Vue.js Tools

  * **Vue.js Core Libraries:**
      * **Vue Router:** For managing navigation within your SPA.
      * **Pinia (or Vuex for Vue 2):** For robust state management (authentication status, user profile, etc.).
  * **API Interaction:**
      * **Axios or Fetch API:** For making HTTP requests (from your backend for OAuth token exchange and from frontend/backend for JSM API calls).
  * **OAuth Client Libraries (for your backend):** Libraries to simplify server-side OAuth 2.0 token exchange for your chosen backend language (e.g., `passport-atlassian-oauth2` for Node.js).
  * **UI Frameworks/Component Libraries (Optional):**
      * Vuetify, BootstrapVue, Quasar, or Tailwind CSS with Headless UI for building the user interface.
  * **Build Tools:**
      * Vite or Vue CLI for project scaffolding and building.

This comprehensive approach provides a secure, standard, and manageable way to build a feature-rich custom JSM portal integrated with your existing Okta identity infrastructure.

```