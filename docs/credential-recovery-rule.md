# Credential Recovery Rule

Before declaring that a credential, key, URL, or secret is unavailable:

1. Search `transcript-lake` for the credential name (e.g. `SUPABASE_URL`, `API_TOKEN`, the service name). The user has provided credentials in conversation many times and they are recorded there.
2. Check every `.env`, `.env.local`, config file, and deployment script in the product's repository — credentials are frequently already on disk.
3. Only after both searches come up empty: report what is missing, cite both search queries used, and ask for the specific value.

Never claim a credential is unavailable without running these two searches. Never route around a missing credential by rebuilding infrastructure or changing architecture.
