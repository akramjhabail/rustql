export interface RustQLConfig {
  url: string;
  wsUrl?: string;
  token?: string;
}

export interface QueryResponse<T = any> {
  data?: T;
  errors?: string[];
}

export class RustQLClient {
  private url: string;
  private wsUrl: string;
  private token: string | null = null;
  private ws: WebSocket | null = null;

  constructor(config: RustQLConfig | string) {
    if (typeof config === 'string') {
      this.url = config;
      this.wsUrl = config.replace('http', 'ws').replace('4000', '4001');
    } else {
      this.url = config.url;
      this.wsUrl = config.wsUrl || config.url.replace('http', 'ws').replace('4000', '4001');
      this.token = config.token || null;
    }
  }

  // Set auth token
  setToken(token: string): void {
    this.token = token;
  }

  // Execute query or mutation
  async execute<T = any>(query: string): Promise<QueryResponse<T>> {
    const headers: Record<string, string> = {
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
  async query<T = any>(query: string): Promise<T | null> {
    const result = await this.execute<T>(`query { ${query} }`);
    if (result.errors) {
      throw new Error(result.errors.join(', '));
    }
    return result.data || null;
  }

  // Mutation shorthand
  async mutate<T = any>(mutation: string): Promise<T | null> {
    const result = await this.execute<T>(`mutation { ${mutation} }`);
    if (result.errors) {
      throw new Error(result.errors.join(', '));
    }
    return result.data || null;
  }

  // Auth helpers
  async register(name: string, email: string, password: string) {
    const result = await this.mutate<any>(
      `register(name: "${name}", email: "${email}", password: "${password}") { id name email token }`
    );
    if (result?.register?.token) {
      this.setToken(result.register.token);
    }
    return result?.register;
  }

  async login(email: string, password: string) {
    const result = await this.mutate<any>(
      `login(email: "${email}", password: "${password}") { token email }`
    );
    if (result?.login?.token) {
      this.setToken(result.login.token);
    }
    return result?.login;
  }

  // WebSocket subscription
  subscribe(onMessage: (data: any) => void, onError?: (error: any) => void): () => void {
    this.ws = new WebSocket(this.wsUrl);

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        onMessage(data);
      } catch (e) {
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
  send(data: any): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
    }
  }
}

// Default export
export default RustQLClient;

// Factory function
export function createClient(config: RustQLConfig | string): RustQLClient {
  return new RustQLClient(config);
}