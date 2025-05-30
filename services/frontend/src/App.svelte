<script lang="ts">
    import { Router, Route } from "svelte-routing";
    import { onMount } from 'svelte';
    import { authStore } from './store/authStore';
    import { isTokenExpired, refreshAccessToken, getUserInfo } from './utils/auth';
    
    import Home from "./routes/Home.svelte";
    import BookPage from "./routes/BookPage.svelte";
    import Header from "./components/Header.svelte";
    import Footer from "./components/Footer.svelte";
    import Callback from "./routes/Callback.svelte";
    import Login from "./routes/Login.svelte";
    import CreateBook from "./routes/CreateBook.svelte";
    import EditBook from "./routes/EditBook.svelte";
    import CreateAuthor from "./routes/CreateAuthor.svelte";
    import EditAuthor from "./routes/EditAuthor.svelte";
    import AuthorPage from "./routes/AuthorPage.svelte";
    import Catalog from "./routes/Catalog.svelte";
    import ChapterEditor from "./routes/ChapterEditor.svelte";
    import ChaptersList from "./routes/ChaptersList.svelte";
    import ChapterReader from "./routes/ChapterReader.svelte";

    export let url = "";
    
    onMount(() => {
        authStore.initialize();
        
        authStore.subscribe(state => {
            if (state.token && isTokenExpired(state.token)) {
                refreshTokens(state.token.refresh_token);
            }
        });
    });
    
    async function refreshTokens(refreshToken: string) {
        try {
            const newToken = await refreshAccessToken(refreshToken);
            authStore.setTokens(newToken.access_token, newToken.refresh_token, newToken.token_type);
            
            const user = await getUserInfo(newToken);
            authStore.setUser(user);
        } catch (e) {
            console.error("Failed to refresh token:", e);
            authStore.logout();
        }
    }
</script>

<Router {url}>
    <div class="app">
        <Header />
        <main>
            <Route path="/" component={Home} />
            <Route path="/catalog" component={Catalog} />
            <Route path="/book/:id" component={BookPage} />
            <Route path="/author/:id" component={AuthorPage} />
            <Route path="/login" component={Login} />
            <Route path="/callback" component={Callback} />
            <Route path="/admin/create-book" component={CreateBook} />
            <Route path="/admin/edit-book/:id" component="{EditBook}" />
            <Route path="/admin/create-author" component="{CreateAuthor}" />
            <Route path="/admin/edit-author/:id" component="{EditAuthor}" />
            <Route path="/book/:bookId/chapters" component="{ChaptersList}" />
            <Route path="/book/:bookId/chapters/new" component="{ChapterEditor}" />
            <Route path="/book/:bookId/chapters/:index/edit" component="{ChapterEditor}" />
            <Route path="/book/:bookId/chapter" component="{ChapterReader}" />
        </main>
        <Footer />
    </div>
</Router>
  
<style>
    :global(:root) {
        --primary-color: #8b5cf6;
        --secondary-color: #4f46e5;
        --dark-bg: #111827;
        --light-bg: #1f2937;
        --text-light: #f3f4f6;
        --text-muted: #9ca3af;
        --border-color: #374151;
    }
    
    :global(body) {
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        background-color: var(--dark-bg);
        color: var(--text-light);
        line-height: 1.6;
        margin: 0;
        padding: 0;
    }
    
    :global(*) {
        box-sizing: border-box;
    }
    
    .app {
        display: flex;
        flex-direction: column;
        min-height: 100vh;
    }
    
    main {
        flex: 1;
    }
</style>