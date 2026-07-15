"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.RustQLClient = void 0;
exports.createClient = createClient;
class RustQLClient {
    constructor(config) {
        this.token = null;
        this.ws = null;
        if (typeof config === 'string') {
            this.url = config;
            this.wsUrl = config.replace('http', 'ws').replace('4000', '4001');
        }
        else {
            this.url = config.url;
            this.wsUrl = config.wsUrl || config.url.replace('http', 'ws').replace('4000', '4001');
            this.token = config.token || null;
        }
    }
    // Set auth token
    setToken(token) {
        this.token = token;
    }
    // Execute query or mutation
    async execute(query) {
        const headers = {
            'Content-Type': 'application/json',
        };
        if (this.token) {
            headers['Authorization'] = `Bearer ${this.token}`;
        }
        const response = await fetch(this.url, {
            method: 'POST',
            headers,
            body: JSON.stringify({ query }),
        });
        return response.json();
    }
    // Query shorthand
    async query(query) {
        const result = await this.execute(`query { ${query} }`);
        if (result.errors) {
            throw new Error(result.errors.join(', '));
        }
        return result.data || null;
    }
    // Mutation shorthand
    async mutate(mutation) {
        const result = await this.execute(`mutation { ${mutation} }`);
        if (result.errors) {
            throw new Error(result.errors.join(', '));
        }
        return result.data || null;
    }
    // Auth helpers
    async register(name, email, password) {
        const result = await this.mutate(`register(name: "${name}", email: "${email}", password: "${password}") { id name email token }`);
        if (result?.register?.token) {
            this.setToken(result.register.token);
        }
        return result?.register;
    }
    async login(email, password) {
        const result = await this.mutate(`login(email: "${email}", password: "${password}") { token email }`);
        if (result?.login?.token) {
            this.setToken(result.login.token);
        }
        return result?.login;
    }
    // WebSocket subscription
    subscribe(onMessage, onError) {
        this.ws = new WebSocket(this.wsUrl);
        this.ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                onMessage(data);
            }
            catch (e) {
                onError?.(e);
            }
        };
        this.ws.onerror = (error) => {
            onError?.(error);
        };
        // Return unsubscribe function
        return () => {
            this.ws?.close();
            this.ws = null;
        };
    }
    // Send WebSocket message
    send(data) {
        if (this.ws?.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(data));
        }
    }
}
exports.RustQLClient = RustQLClient;
// Default export
exports.default = RustQLClient;
// Factory function
function createClient(config) {
    return new RustQLClient(config);
}
