<script lang="ts">
    import { onMount } from 'svelte';

    const onLogout = async () => {
        const result = await fetch('/api/auth/logout', {
            method: 'GET',
        })
        if (result.ok) {
            window.location.href = '/'
        }
    }

    export interface Tweet {
        id: string;
        username: string;
        content: string;
        created_at: string;
    }

    let tweets: Tweet[] = [];
    let newTweetContent: string = '';
    let showModal: boolean = false;
    let me: Me = {
        id: '',
        username: '',
        email: '',
        password: '',
        createdat: ''
    };

    onMount(async () => {
        await fetchTweets();
        await fetchMe();
    });

    const fetchTweets = async () => {
        const result = await fetch('/api/timeline', {
            method: 'GET',
        });
        if (result.ok) {
            tweets = await result.json();
        } else {
            console.log('error: failed to fetch timeline');
        }
    }

    interface Me {
        id: string;
        username: string;
        email: string;
        password: string; //空文字で返ってくる
        createdat: string;

    }
    const fetchMe = async () => {
        const result = await fetch('/api/auth/me', {
            method: 'GET',
        });
        if (result.ok) {
            me = await result.json();
        } else {
            console.log("error: failed to fetch user info")
        }

    }


    const postTweet = async () => {
        if (!newTweetContent) {
            return;
        }
        const result = await fetch('/api/content/user_tweets', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ content: newTweetContent }),
        });
        if (result.ok) {
            newTweetContent = '';
            await fetchTweets();
            showModal = false;
        } else {
            console.log('error: failed to post tweet');
        }
    }

    const toggleModal = () => {
        showModal = !showModal;
    }

</script>
<header class="sticky top-0 z-50 flex justify-between items-center bg-white p-4 border-b border-gray-200">
    <h1 class="text-3xl font-bold">Sveltter</h1>
    <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded" on:click={onLogout}>Logout</button>
</header>
<div class="bg-gray-50 min-h-screen flex">
    <aside class="w-1/4 bg-white border-r border-gray-200 p-4">
        <h2 class="text-2xl font-bold mb-4">Profile</h2>
        <div>
            <h3 class="font-bold mb-2">Username</h3>
            <p>{me.username}</p>
        </div>
        <div class="mt-4">
            <h3 class="font-bold mb-2">Email</h3>
            <p>{me.email}</p>
        </div>
    </aside>
    <main class="w-3/4">

        <h1 class="text-3xl font-bold text-center my-8">Timeline</h1>
        <div class="tweet-list max-w-lg mx-auto">
            {#each tweets as tweet (tweet.id)}
                <div class="tweet p-4 bg-white mb-4 rounded shadow">
                    <h2 class="text-lg font-bold mb-2">{tweet.username}</h2>
                    <p class="mb-2">{tweet.content}</p>
                    <small class="text-gray-500">{tweet.created_at}</small>
                </div>
            {/each}
        </div>
        <button class="fixed text-white z-50 bottom-10 right-10 py-4 px-8 border-2 bg-blue-400 rounded-full cursor-pointer" on:click={toggleModal}>Tweet</button>
        {#if showModal}
            <div class="fixed top-0 left-0 w-full h-full flex items-center justify-center" style="background-color: rgba(0, 0, 0, 0.5);">
                <div class="bg-white rounded p-8">
                    <h2 class="text-2xl font-bold mb-4">New Tweet</h2>
                    <input class="border border-gray-400 rounded w-full py-2 px-4 mb-4" type="text" bind:value={newTweetContent} placeholder="What's happening?">
                    <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded w-full" type="button" on:click={postTweet}>Post Tweet</button>
                </div>
            </div>
        {/if}
    </main>
</div>
