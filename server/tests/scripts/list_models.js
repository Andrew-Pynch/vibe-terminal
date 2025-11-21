const https = require('https');
const fs = require('fs');
const path = require('path');

// Try to read .env from repo root (../../../../.env from this script's perspective)
// Script is in server/tests/scripts/
const rootEnv = path.join(__dirname, '../../../.env');
if (fs.existsSync(rootEnv)) {
    const envConfig = fs.readFileSync(rootEnv, 'utf8');
    envConfig.split('\n').forEach(line => {
        const match = line.match(/^([^=]+)=(.*)$/);
        if (match) {
            const key = match[1].trim();
            const value = match[2].trim();
            if (!process.env[key]) {
                process.env[key] = value;
            }
        }
    });
}

const API_KEY = process.env.GEMINI_API_KEY;
if (!API_KEY) {
    console.error("GEMINI_API_KEY env var required");
    process.exit(1);
}

const url = `https://generativelanguage.googleapis.com/v1beta/models?key=${API_KEY}`;

https.get(url, (res) => {
    let data = '';
    res.on('data', chunk => data += chunk);
    res.on('end', () => {
        try {
            const json = JSON.parse(data);
            if (json.models) {
                console.log("Available Models:");
                json.models.forEach(m => {
                    if (m.supportedGenerationMethods && m.supportedGenerationMethods.includes("generateContent")) {
                        console.log(`- ${m.name}`);
                    }
                });
            } else {
                console.log("Error response:", JSON.stringify(json, null, 2));
            }
        } catch (e) {
            console.error("Failed to parse:", e);
            console.log("Raw data:", data);
        }
    });
});
