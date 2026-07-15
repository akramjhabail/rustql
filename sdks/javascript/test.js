const { RustQLClient } = require('./dist/index.js');

async function runTests() {
    console.log('🦀 RustQL JS SDK Tests\n');
    
    const client = new RustQLClient('http://localhost:4000/rustql');

    try {
        // Test 1 — Hello Query
        console.log('Test 1: Hello Query');
        const hello = await client.execute('query { hello }');
        console.log('✅', JSON.stringify(hello.data));

        // Test 2 — Users Query
        console.log('\nTest 2: Users Query');
        const users = await client.execute('query { users { id name email } }');
        console.log('✅', JSON.stringify(users.data));

        // Test 3 — Register
        console.log('\nTest 3: Register');
        const reg = await client.register('SDKUser', 'sdk@rustql.dev', 'password123');
        console.log('✅ Registered:', reg?.name, '| Token:', reg?.token?.substring(0, 20) + '...');

        // Test 4 — Login
        console.log('\nTest 4: Login');
        const login = await client.login('sdk@rustql.dev', 'password123');
        console.log('✅ Logged in:', login?.email);

        console.log('\n🎉 All tests passed!');

    } catch (error) {
        console.error('❌ Error:', error.message);
    }
}

runTests();