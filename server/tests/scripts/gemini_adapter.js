#!/usr/bin/env node
/**
 * gemini_adapter.js
 * A simple agent that reads INSTRUCTION.md, queries Google Gemini via HTTP,
 * and writes the response to RESULT.md.
 */
const fs = require('fs');
const https = require('https');
const path = require('path');

const WORK_DIR = process.cwd();
const DEBUG_LOG = path.join(WORK_DIR, "debug_log.txt");

function log(msg) {
    const timestamp = new Date().toISOString();
    const logMsg = `[${timestamp}] ${msg}\n`;
    fs.appendFileSync(DEBUG_LOG, logMsg);
    console.log(msg);
}

function error(msg) {
    const timestamp = new Date().toISOString();
    const logMsg = `[${timestamp}] ERROR: ${msg}\n`;
    fs.appendFileSync(DEBUG_LOG, logMsg);
    console.error(msg);
}

log("Starting Gemini Adapter...");
log(`CWD: ${WORK_DIR}`);

const API_KEY = process.env.GEMINI_API_KEY;
const MODEL = "gemini-2.0-flash"; 

if (!API_KEY) {
    error("GEMINI_API_KEY environment variable not set.");
    process.exit(1);
}

const INSTRUCTION_FILE = path.join(WORK_DIR, "INSTRUCTION.md");
const RESULT_FILE = path.join(WORK_DIR, "RESULT.md");

// 1. Read Instruction
if (!fs.existsSync(INSTRUCTION_FILE)) {
    error(`${INSTRUCTION_FILE} not found.`);
    process.exit(1);
}

const instruction = fs.readFileSync(INSTRUCTION_FILE, 'utf8');
log(`Read instruction: ${instruction}`);

// 2. Construct Prompt
// We use the instruction directly. The caller is responsible for system prompting.
const prompt = instruction;

// 3. Call Gemini API
const url = `https://generativelanguage.googleapis.com/v1beta/models/${MODEL}:generateContent?key=${API_KEY}`;
const requestBody = JSON.stringify({
    contents: [{
        parts: [{ text: prompt }]
    }]
});

log("Calling Gemini API...");

const req = https.request(url, {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json'
    }
}, (res) => {
    let data = '';
    res.on('data', (chunk) => data += chunk);
    res.on('end', () => {
        if (res.statusCode !== 200) {
            error(`API Error ${res.statusCode}: ${data}`);
            process.exit(1);
        }

        try {
            const response = JSON.parse(data);
            const text = response.candidates?.[0]?.content?.parts?.[0]?.text;
            
            if (!text) {
                error("No text in response: " + data);
                process.exit(1);
            }

            // 4. Write Result
            fs.writeFileSync(RESULT_FILE, text);
            log(`Success! Result written to ${RESULT_FILE}`);

            // 5. Check for Task Graph (Heuristic)
            try {
                // Extract JSON from code block if present
                let jsonText = text;
                const jsonBlockRegex = /```json\s*([\s\S]*?)\s*```/;
                const match = text.match(jsonBlockRegex);
                if (match) {
                    jsonText = match[1];
                }

                // Heuristic: Check for "tasks" array
                if (jsonText.includes("tasks")) {
                    const jsonObj = JSON.parse(jsonText);
                    if (jsonObj && Array.isArray(jsonObj.tasks)) {
                        log("Detected TaskGraph in output. Writing to TASK_GRAPH.json...");
                        const taskGraphPath = path.join(WORK_DIR, "TASK_GRAPH.json");
                        fs.writeFileSync(taskGraphPath, JSON.stringify(jsonObj, null, 2));
                        log(`Success! TaskGraph written to ${taskGraphPath}`);
                    }
                }
            } catch (e) {
                // Not JSON or not a task graph, ignore
                // log(`JSON parse check failed: ${e.message}`);
            }

        } catch (e) {
            error("Failed to parse JSON response from API: " + e.message);
            process.exit(1);
        }
    });
});

req.on('error', (e) => {
    error(`Request error: ${e.message}`);
    process.exit(1);
});

req.write(requestBody);
req.end();