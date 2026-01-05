# macOS Notarization Setup

This document describes the procedure for setting up macOS code signing and notarization for Eyes on Claude Code.

## Prerequisites

- Apple Developer Program membership
- Developer ID Application certificate
- App Store Connect API key

## Step 1: Export Certificate

1. Open Keychain Access
2. Find "Developer ID Application: Your Name (TEAM_ID)" certificate
3. Right-click and select "Export..."
4. Save as `.p12` file with a strong password

## Step 2: Create App Store Connect API Key

1. Go to [App Store Connect](https://appstoreconnect.apple.com/)
2. Navigate to Users and Access > Integrations > App Store Connect API
3. Click "+" to create a new key
4. Name: `notarization` (or any descriptive name)
5. Access: `Developer`
6. Download the `.p8` file (only available once)
7. Note down the Key ID and Issuer ID

## Step 3: Configure GitHub Secrets

Go to your repository Settings > Secrets and variables > Actions, and add the following secrets:

### `APPLE_CERTIFICATE`

Base64-encoded `.p12` certificate:

```bash
base64 -i certificate.p12 | pbcopy
```

### `APPLE_CERTIFICATE_PASSWORD`

The password you set when exporting the `.p12` file.

### `APPLE_SIGNING_IDENTITY`

The full name of your signing certificate. Format:

```
Developer ID Application: Your Name (TEAM_ID)
```

To find this, run:

```bash
security find-identity -v -p codesigning
```

### `APPLE_API_KEY`

The contents of the `.p8` file:

```bash
cat AuthKey_XXXXXXXXXX.p8 | pbcopy
```

### `APPLE_API_KEY_ID`

The Key ID shown in App Store Connect (e.g., `XXXXXXXXXX`).

### `APPLE_API_ISSUER`

The Issuer ID shown in App Store Connect (e.g., `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).

## Verification

After setting up all secrets, trigger a release workflow. The build logs should show:

1. Certificate import succeeding
2. Code signing with your Developer ID
3. Notarization submission to Apple
4. Notarization approval (stapling)

## Troubleshooting

### "No identity found"

- Ensure `APPLE_SIGNING_IDENTITY` matches exactly the certificate name
- Verify the certificate was imported correctly

### "Unable to authenticate"

- Check that `APPLE_API_KEY`, `APPLE_API_KEY_ID`, and `APPLE_API_ISSUER` are correct
- Ensure the API key has sufficient permissions

### Notarization fails

- Check Apple's notarization logs (URL provided in error message)
- Common issues: hardened runtime not enabled, unsigned libraries

## Security Notes

- Never commit certificates or API keys to the repository
- Rotate API keys periodically
- Use GitHub's secret scanning to prevent accidental exposure
