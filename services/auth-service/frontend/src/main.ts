import App from './App.svelte';

const app = new App({
    target: document.body,
    props: {
        logoPath: "https://tobyharriman.com/wp-content/uploads/2012/10/adobe-logo-png-02611.png",
        primaryColor: "#E23127",
        AppName: "Lorem Ipsum",
    }
});

export default app;