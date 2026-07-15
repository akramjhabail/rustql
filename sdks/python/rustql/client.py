import requests
import json
import threading
from typing import Any, Dict, Optional, Callable
try:
    import websocket
    WS_AVAILABLE = True
except ImportError:
    WS_AVAILABLE = False


class RustQLError(Exception):
    """RustQL API Error"""
    pass


class RustQLClient:
    """
    🦀 RustQL Python Client
    
    Usage:
        client = RustQLClient("http://localhost:4000/rustql")
        result = client.query("{ users { name email } }")
    """

    def __init__(self, url: str, token: Optional[str] = None):
        self.url = url
        self.ws_url = url.replace("http", "ws").replace("4000", "4001")
        self.token = token
        self.session = requests.Session()
        self._ws = None

    def set_token(self, token: str) -> None:
        """Set auth token"""
        self.token = token
        self.session.headers.update({
            "Authorization": f"Bearer {token}"
        })

    def execute(self, query: str) -> Dict[str, Any]:
        """Execute a raw query or mutation"""
        headers = {"Content-Type": "application/json"}
        if self.token:
            headers["Authorization"] = f"Bearer {self.token}"

        response = self.session.post(
            self.url,
            json={"query": query},
            headers=headers
        )
        response.raise_for_status()
        return response.json()

    def query(self, query: str) -> Any:
        """Execute a query"""
        result = self.execute(f"query {{ {query} }}")
        if result.get("errors"):
            raise RustQLError(", ".join(result["errors"]))
        return result.get("data")

    def mutate(self, mutation: str) -> Any:
        """Execute a mutation"""
        result = self.execute(f"mutation {{ {mutation} }}")
        if result.get("errors"):
            raise RustQLError(", ".join(result["errors"]))
        return result.get("data")

    def register(self, name: str, email: str, password: str) -> Dict:
        """Register a new user"""
        result = self.mutate(
            f'register(name: "{name}", email: "{email}", password: "{password}") '
            f'{{ id name email token }}'
        )
        if result and result.get("register", {}).get("token"):
            self.set_token(result["register"]["token"])
        return result.get("register", {}) if result else {}

    def login(self, email: str, password: str) -> Dict:
        """Login user"""
        result = self.mutate(
            f'login(email: "{email}", password: "{password}") '
            f'{{ token email }}'
        )
        if result and result.get("login", {}).get("token"):
            self.set_token(result["login"]["token"])
        return result.get("login", {}) if result else {}

    def get_users(self) -> list:
        """Get all users"""
        result = self.query("users { id name email }")
        return result.get("users", []) if result else []

    def create_user(self, name: str, email: str) -> Dict:
        """Create a new user"""
        result = self.mutate(
            f'createUser(name: "{name}", email: "{email}") '
            f'{{ id name email }}'
        )
        return result.get("createUser", {}) if result else {}

    def update_user(self, user_id: int, name: str = None, email: str = None) -> Dict:
        """Update a user"""
        args = [f"id: {user_id}"]
        if name:
            args.append(f'name: "{name}"')
        if email:
            args.append(f'email: "{email}"')

        result = self.mutate(
            f'updateUser({", ".join(args)}) {{ id name email }}'
        )
        return result.get("updateUser", {}) if result else {}

    def delete_user(self, user_id: int) -> str:
        """Delete a user"""
        result = self.mutate(f"deleteUser(id: {user_id})")
        return result.get("deleteUser", "") if result else ""

    def subscribe(
        self,
        on_message: Callable,
        on_error: Optional[Callable] = None
    ) -> None:
        """Subscribe to WebSocket events"""
        if not WS_AVAILABLE:
            raise RustQLError("websocket-client not installed. Run: pip install websocket-client")

        def on_ws_message(ws, message):
            try:
                data = json.loads(message)
                on_message(data)
            except Exception as e:
                if on_error:
                    on_error(e)

        def on_ws_error(ws, error):
            if on_error:
                on_error(error)

        self._ws = websocket.WebSocketApp(
            self.ws_url,
            on_message=on_ws_message,
            on_error=on_ws_error
        )

        thread = threading.Thread(
            target=self._ws.run_forever
        )
        thread.daemon = True
        thread.start()

    def send(self, data: Any) -> None:
        """Send WebSocket message"""
        if self._ws:
            self._ws.send(json.dumps(data))

    def close(self) -> None:
        """Close WebSocket connection"""
        if self._ws:
            self._ws.close()