<script lang="ts">
    let userName: string = '';
    let email: string = '';
    let password: string = '';

    const handleUserNameChange = (event: InputEvent) => {
        userName = (event.target as HTMLInputElement).value;
    };

    const handleEmailChange = (event: InputEvent) => {
        email = (event.target as HTMLInputElement).value;
    };

    const handlePasswordChange = (event: InputEvent) => {
        password = (event.target as HTMLInputElement).value;
    };

    const onSubmit = async (event: Event) => {
        event.preventDefault();

        try {
            const response = await fetch('/api/auth/register', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    username: userName,
                    email: email,
                    password: password,
                })
            });

            if (response.ok) {
                console.log('register successful');
                window.location.href = '/login';
            } else {
                console.log('register failed');
            }
        } catch (error) {
            console.error('An error occurred:', error);
        }
    };
</script>

<div class="flex flex-col items-center justify-center min-h-screen bg-gray-100">
    <div class="bg-white p-6 rounded-lg shadow-lg w-80">
        <h2 class="tesxt-2xl font-bold mb-5 text-gray-800">Create an account</h2>
        <form class="space-y-5" on:submit={onSubmit}>
            <div>
                <label for="username" class="block mb-1 font-bold text-gray-500">User name</label>
                <input id="username" type="text" on:change={handleUserNameChange} class="w-full border-2 border-gray-200 p-3 rounded outline-none focus:border-blue-500" />
            </div>
            <div>
                <label for="email" class="block mb-1 font-bold text-gray-500">Email</label>
                <input id="email" type="text" on:change={handleEmailChange} class="w-full border-2 border-gray-200 p-3 rounded outline-none focus:border-blue-500" />
            </div>
            <div>
                <label for="password" class="block mb-1 font-bold text-gray-500">Password</label>
                <input id="password" type="password" on:change={handlePasswordChange} class="w-full border-2 border-gray-200 p-3 rounded outline-none focus:border-blue-500" />
            </div>
            <div>
                <button type="submit" class="w-full py-3 px-4 bg-blue-600 hover:bg-blue-700 rounded-md text-white font-bold">Login</button>
            </div>
        </form>
    </div>
</div>
