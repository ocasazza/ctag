# E2E Test Structure

The following approach is used for E2E tests:

1. Verify that the `ctag get` command returns the expected data using the Atlassian API.

2. Verify that the `ctag add`, `ctag remove`, and `ctag replace` commands adds the expected tags to the expected pages using the verified `ctag get` command in parallel and the Atlassian API.

3. Verify that the advanced and bulk usage using the verified add remove and replace commands and the Atlassian API.
