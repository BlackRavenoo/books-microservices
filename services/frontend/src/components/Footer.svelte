<script lang="ts">
    import { link } from 'svelte-routing';
</script>
  
<footer>
    <div class="container">
        <div class="footer-links">
            <a href="/about" use:link class="footer-link">О нас</a>
            <a href="/rules" use:link class="footer-link">Правила</a>
            <a href="/contacts" use:link class="footer-link">Контакты</a>
            <a href="/help" use:link class="footer-link">Помощь</a>
        </div>
        <div class="copyright">
            © {new Date().getFullYear()} BooksLib. Все права защищены.
        </div>
    </div>
</footer>
  
<style>
    footer {
        background-color: var(--light-bg);
        border-top: 1px solid var(--border-color);
        padding: 2rem 0;
        margin-top: 2rem;
    }
    
    .container {
        width: 100%;
        max-width: 1200px;
        margin: 0 auto;
        padding: 0 1rem;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }
    
    .footer-links {
        display: flex;
        gap: 1rem;
    }
    
    .footer-link {
        color: var(--text-muted);
        text-decoration: none;
        font-size: 0.875rem;
        transition: color 0.2s;
    }
    
    .footer-link:hover {
        color: var(--primary-color);
    }
    
    .copyright {
        color: var(--text-muted);
        font-size: 0.875rem;
    }
</style>