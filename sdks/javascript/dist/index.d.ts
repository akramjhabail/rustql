export interface RustQLConfig {
    url: string;
    wsUrl?: string;
    token?: string;
}
export interface QueryResponse<T = any> {
    data?: T;
    errors?: string[];
}
export declare class RustQLClient {
    private url;
    private wsUrl;
    private token;
    private ws;
    constructor(config: RustQLConfig | string);
    setToken(token: string): void;
    execute<T = any>(query: string): Promise<QueryResponse<T>>;
    query<T = any>(query: string): Promise<T | null>;
    mutate<T = any>(mutation: string): Promise<T | null>;
    register(name: string, email: string, password: string): Promise<any>;
    login(email: string, password: string): Promise<any>;
    subscribe(onMessage: (data: any) => void, onError?: (error: any) => void): () => void;
    send(data: any): void;
}
export default RustQLClient;
export declare function createClient(config: RustQLConfig | string): RustQLClient;
