from rustql import RustQLClient, RustQLError

def run_tests():
    print("🦀 RustQL Python SDK Tests\n")
    
    client = RustQLClient("http://localhost:4000/rustql")
    
    try:
        # Test 1 — Hello Query
        print("Test 1: Hello Query")
        result = client.execute("query { hello }")
        print("✅", result["data"])

        # Test 2 — Users Query
        print("\nTest 2: Users Query")
        users = client.get_users()
        print(f"✅ Found {len(users)} users")
        for user in users[:3]:
            print(f"   - {user['name']} ({user['email']})")

        # Test 3 — Register
        print("\nTest 3: Register")
        user = client.register("PythonUser", "python@rustql.dev", "password123")
        print(f"✅ Registered: {user.get('name')} | Token: {user.get('token', '')[:20]}...")

        # Test 4 — Login
        print("\nTest 4: Login")
        login = client.login("python@rustql.dev", "password123")
        print(f"✅ Logged in: {login.get('email')}")

        # Test 5 — Create User
        print("\nTest 5: Create User")
        new_user = client.create_user("PyUser", "pyuser@rustql.dev")
        print(f"✅ Created: {new_user.get('name')} (id: {new_user.get('id')})")

        # Test 6 — Update User
        print("\nTest 6: Update User")
        if new_user.get('id'):
            updated = client.update_user(
                int(new_user['id']),
                name="PyUser Updated"
            )
            print(f"✅ Updated: {updated.get('name')}")

        # Test 7 — Delete User
        print("\nTest 7: Delete User")
        if new_user.get('id'):
            deleted = client.delete_user(int(new_user['id']))
            print(f"✅ {deleted}")

        print("\n🎉 All tests passed!")

    except RustQLError as e:
        print(f"❌ RustQL Error: {e}")
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    run_tests()