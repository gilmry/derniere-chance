<div class="screen">
  <div class="hero">
    <div class="hero-emoji" role="img" aria-label="Panier de fruits et légumes">🧺</div>
    <h1>Sauve un panier,<br />régale-toi malin.</h1>
    <p class="lead">
      Repère les commerçants proches de chez toi et récupère leurs invendus à petit prix.
    </p>
  </div>
  <div class="actions">
    <a class="btn btn-primary" href="/compte?mode=register">Créer un compte</a>
    <a class="btn btn-secondary" href="/compte?mode=login">J'ai déjà un compte</a>
    <a class="merchant-link" href="/pro/login?mode=register">Vous êtes commerçant ? Créer mon compte pro</a>
    <div class="dots">
      <span class="dot dot--active"></span>
      <span class="dot"></span>
      <span class="dot"></span>
    </div>
  </div>
</div>

<style>
  .screen {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 28px 24px;
    gap: 24px;
  }

  .hero {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    text-align: center;
  }

  .hero-emoji {
    font-size: 110px;
    line-height: 1;
  }

  h1 {
    font-size: 26px;
  }

  .lead {
    font-size: 15px;
    color: var(--color-muted);
    line-height: 1.5;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .merchant-link {
    text-align: center;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-muted);
    padding: 4px;
  }

  .dots {
    display: flex;
    justify-content: center;
    gap: 6px;
    padding-top: 4px;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 3px;
    background: var(--color-border);
  }

  .dot--active {
    width: 20px;
    background: var(--color-primary);
  }
</style>
