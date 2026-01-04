import { Component, OnInit, signal, AfterViewInit, ElementRef, ViewChild, HostListener, OnDestroy } from '@angular/core';
import { RouterOutlet } from '@angular/router';

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  size: number;
  alpha: number;
  color: string;
}

@Component({
  selector: 'app-root',
  imports: [RouterOutlet],
  templateUrl: './app.html',
  styleUrl: './app.css'
})
export class App implements OnInit, AfterViewInit, OnDestroy {
  @ViewChild('starsContainer') starsContainer!: ElementRef<HTMLDivElement>;
  @ViewChild('particleCanvas') particleCanvas!: ElementRef<HTMLCanvasElement>;
  
  protected readonly activeTab = signal<'apk' | 'source'>('apk');
  protected readonly copied = signal(false);
  protected readonly isScrolled = signal(false);
  
  private particles: Particle[] = [];
  private animationId: number | null = null;
  private ctx: CanvasRenderingContext2D | null = null;

  ngOnInit(): void {
    // Initialize on component creation
  }

  ngAfterViewInit(): void {
    this.createStars();
    this.initParticles();
  }

  ngOnDestroy(): void {
    if (this.animationId) {
      cancelAnimationFrame(this.animationId);
    }
  }

  @HostListener('window:scroll')
  onScroll(): void {
    this.isScrolled.set(window.scrollY > 50);
  }
  
  @HostListener('window:resize')
  onResize(): void {
    if (this.particleCanvas) {
      const canvas = this.particleCanvas.nativeElement;
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    }
  }

  setActiveTab(tab: 'apk' | 'source'): void {
    this.activeTab.set(tab);
  }

  async copyCode(): Promise<void> {
    const code = `# Clone the repository
git clone https://github.com/quinnjr/fgo-sheba.git
cd fgo-sheba

# Install Rust targets for Android
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi

# Build Rust libraries
cargo build --release --target aarch64-linux-android

# Build Android APK
cd android
./gradlew assembleRelease`;

    try {
      await navigator.clipboard.writeText(code);
      this.copied.set(true);
      setTimeout(() => this.copied.set(false), 2000);
    } catch {
      console.error('Failed to copy code');
    }
  }

  private createStars(): void {
    if (!this.starsContainer) return;
    
    const container = this.starsContainer.nativeElement;
    const starCount = 150;
    
    for (let i = 0; i < starCount; i++) {
      const star = document.createElement('div');
      star.className = 'star';
      star.style.left = `${Math.random() * 100}%`;
      star.style.top = `${Math.random() * 100}%`;
      star.style.setProperty('--duration', `${2 + Math.random() * 4}s`);
      star.style.setProperty('--delay', `${Math.random() * 3}s`);
      star.style.setProperty('--opacity', `${0.3 + Math.random() * 0.7}`);
      
      // Some stars are bigger
      if (Math.random() > 0.9) {
        star.style.width = '3px';
        star.style.height = '3px';
        star.style.boxShadow = '0 0 6px 2px rgba(255, 255, 255, 0.3)';
      }
      
      container.appendChild(star);
    }
  }
  
  private initParticles(): void {
    if (!this.particleCanvas) return;
    
    const canvas = this.particleCanvas.nativeElement;
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    this.ctx = canvas.getContext('2d');
    
    if (!this.ctx) return;
    
    // Create initial particles
    const colors = ['#ffd700', '#9370db', '#4169e1', '#b19cd9'];
    for (let i = 0; i < 50; i++) {
      this.particles.push({
        x: Math.random() * canvas.width,
        y: Math.random() * canvas.height,
        vx: (Math.random() - 0.5) * 0.3,
        vy: (Math.random() - 0.5) * 0.3,
        size: Math.random() * 3 + 1,
        alpha: Math.random() * 0.4 + 0.1,
        color: colors[Math.floor(Math.random() * colors.length)]
      });
    }
    
    this.animateParticles();
  }
  
  private animateParticles(): void {
    if (!this.ctx || !this.particleCanvas) return;
    
    const canvas = this.particleCanvas.nativeElement;
    this.ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    this.particles.forEach(p => {
      // Update position
      p.x += p.vx;
      p.y += p.vy;
      
      // Wrap around edges
      if (p.x < 0) p.x = canvas.width;
      if (p.x > canvas.width) p.x = 0;
      if (p.y < 0) p.y = canvas.height;
      if (p.y > canvas.height) p.y = 0;
      
      // Draw particle
      this.ctx!.beginPath();
      this.ctx!.arc(p.x, p.y, p.size, 0, Math.PI * 2);
      this.ctx!.fillStyle = p.color;
      this.ctx!.globalAlpha = p.alpha;
      this.ctx!.fill();
      
      // Draw glow
      this.ctx!.beginPath();
      this.ctx!.arc(p.x, p.y, p.size * 2, 0, Math.PI * 2);
      const gradient = this.ctx!.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.size * 2);
      gradient.addColorStop(0, p.color);
      gradient.addColorStop(1, 'transparent');
      this.ctx!.fillStyle = gradient;
      this.ctx!.globalAlpha = p.alpha * 0.3;
      this.ctx!.fill();
    });
    
    this.ctx.globalAlpha = 1;
    this.animationId = requestAnimationFrame(() => this.animateParticles());
  }
}
