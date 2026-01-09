// AudioMultiverse CI/CD Pipeline
// Jenkins läuft auf Windows Server
// Benötigte Agents: windows, linux

pipeline {
    agent none
    
    environment {
        APP_NAME = 'AudioMultiverse'
        APP_VERSION = '0.1.0'
        RUST_VERSION = '1.75.0'
        NODE_VERSION = '20'
    }
    
    options {
        buildDiscarder(logRotator(numToKeepStr: '10'))
        timestamps()
        timeout(time: 2, unit: 'HOURS')
    }
    
    stages {
        
        // ============================================================
        // STAGE 1: Checkout
        // ============================================================
        
        stage('Checkout') {
            agent { label 'windows' }
            steps {
                checkout scm
                stash includes: '**', name: 'source'
            }
        }
        
        // ============================================================
        // STAGE 2: Parallele Builds
        // ============================================================
        
        stage('Build All') {
            parallel {
                
                // --------------------------------------------------------
                // Windows App (MSI + NSIS)
                // --------------------------------------------------------
                
                stage('Windows App') {
                    agent { label 'windows' }
                    
                    steps {
                        unstash 'source'
                        
                        echo '=== Installing Dependencies ==='
                        bat '''
                            @echo off
                            setlocal enabledelayedexpansion
                            
                            REM Rust und MinGW zum PATH hinzufuegen
                            set "PATH=%USERPROFILE%\\.cargo\\bin;C:\\msys64\\mingw64\\bin;%PATH%"
                            
                            REM Node.js pruefen
                            where node >nul 2>&1
                            if errorlevel 1 (
                                echo Node.js nicht gefunden!
                                exit /b 1
                            )
                            
                            REM Rust pruefen und ggf. installieren
                            where rustc >nul 2>&1
                            if errorlevel 1 (
                                echo Rust nicht gefunden - installiere Rust...
                                curl -sSf -o rustup-init.exe https://win.rustup.rs/x86_64
                                rustup-init.exe -y --default-toolchain stable-x86_64-pc-windows-gnu
                                del rustup-init.exe
                                set "PATH=%USERPROFILE%\\.cargo\\bin;%PATH%"
                            )
                            
                            rustc --version
                            cargo --version
                            
                            REM Dependencies installieren
                            cd app
                            call npm install
                        '''
                        
                        echo '=== Building Windows App ==='
                        dir('app') {
                            bat '''
                                @echo off
                                set "PATH=%USERPROFILE%\\.cargo\\bin;C:\\msys64\\mingw64\\bin;%PATH%"
                                call npx tauri build
                            '''
                        }
                        
                        echo '=== Collecting Windows App Artifacts ==='
                        bat '''
                            @echo off
                            mkdir dist\\windows\\app 2>nul
                            xcopy /Y app\\src-tauri\\target\\release\\bundle\\msi\\*.msi dist\\windows\\app\\
                            xcopy /Y app\\src-tauri\\target\\release\\bundle\\nsis\\*.exe dist\\windows\\app\\
                        '''
                        
                        archiveArtifacts artifacts: 'dist/windows/app/*', fingerprint: true
                        stash includes: 'dist/windows/app/*', name: 'windows-app'
                    }
                    
                    post {
                        always {
                            cleanWs()
                        }
                    }
                }
                
                // --------------------------------------------------------
                // Windows Server
                // --------------------------------------------------------
                
                stage('Windows Server') {
                    agent { label 'windows' }
                    
                    steps {
                        unstash 'source'
                        
                        echo '=== Building Windows Server ==='
                        dir('server') {
                            bat '''
                                @echo off
                                set "PATH=%USERPROFILE%\\.cargo\\bin;C:\\msys64\\mingw64\\bin;%PATH%"
                                cargo build --release
                            '''
                        }
                        
                        echo '=== Collecting Server Artifacts ==='
                        bat '''
                            @echo off
                            mkdir dist\\\\windows\\\\server 2>nul
                            xcopy /Y server\\\\target\\\\release\\\\audiomultiverse-server.exe dist\\\\windows\\\\server\\\\
                            xcopy /Y server\\\\config.toml.example dist\\\\windows\\\\server\\\\
                        '''
                        
                        archiveArtifacts artifacts: 'dist/windows/server/*', fingerprint: true
                        stash includes: 'dist/windows/server/*', name: 'windows-server'
                    }
                    
                    post {
                        always {
                            cleanWs()
                        }
                    }
                }
                
                // --------------------------------------------------------
                // Linux Server & App (.deb Pakete)
                // HINWEIS: Erfordert einen Linux-Agent mit Label 'linux'
                // --------------------------------------------------------
                
                stage('Linux Packages') {
                    agent { label 'linux' }
                    
                    options {
                        skipDefaultCheckout()
                    }
                    
                    steps {
                        unstash 'source'
                        
                        echo '=== Installing Dependencies ==='
                        sh '''
                            # Rust
                            command -v rustc || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                            source "$HOME/.cargo/env"
                            
                            # System Dependencies für Tauri
                            sudo apt-get update
                            sudo apt-get install -y \
                                build-essential \
                                libwebkit2gtk-4.1-dev \
                                libappindicator3-dev \
                                librsvg2-dev \
                                patchelf \
                                libssl-dev \
                                libasound2-dev \
                                libjack-jackd2-dev
                        '''
                        
                        echo '=== Building Server ==='
                        dir('server') {
                            sh '''
                                source "$HOME/.cargo/env"
                                cargo build --release
                            '''
                        }
                        
                        echo '=== Building App ==='
                        dir('app') {
                            sh '''
                                npm install
                                source "$HOME/.cargo/env"
                                npx tauri build --bundles deb
                            '''
                        }
                        
                        echo '=== Creating Server .deb Package ==='
                        sh '''
                            source "$HOME/.cargo/env"
                            cargo install cargo-deb || true
                            
                            cd server
                            cargo deb --output ../dist/linux/
                        '''
                        
                        echo '=== Collecting Artifacts ==='
                        sh '''
                            mkdir -p dist/linux
                            
                            # Tauri .deb Pakete sammeln
                            find app/src-tauri/target/release/bundle/deb -name "*.deb" -exec cp {} dist/linux/ \\;
                        '''
                        
                        archiveArtifacts artifacts: 'dist/linux/*.deb', fingerprint: true
                        stash includes: 'dist/linux/*.deb', name: 'linux-packages'
                    }
                    
                    post {
                        always {
                            cleanWs()
                        }
                    }
                }
            }
        }
        
        // ============================================================
        // STAGE 3: Artifacts sammeln
        // ============================================================
        
        stage('Collect Artifacts') {
            agent { label 'windows' }
            
            steps {
                script {
                    // Alle Artifacts sammeln
                    try { unstash 'windows-app' } catch (e) { echo "Windows App: ${e}" }
                    try { unstash 'windows-server' } catch (e) { echo "Windows Server: ${e}" }
                    try { unstash 'linux-packages' } catch (e) { echo "Linux: ${e}" }
                }
                
                echo '=== All Artifacts ==='
                bat 'dir /s /b dist'
                
                // Release-Ordner erstellen
                bat "mkdir release\\${APP_VERSION} 2>nul & xcopy /E /Y dist\\* release\\${APP_VERSION}\\"
                
                archiveArtifacts artifacts: 'release/**/*', fingerprint: true
            }
        }
    }
    
    // ============================================================
    // POST-ACTIONS
    // ============================================================
    
    post {
        success {
            echo 'Build erfolgreich!'
        }
        
        failure {
            echo 'Build fehlgeschlagen!'
        }
        
        always {
            echo "Pipeline beendet: ${currentBuild.result}"
        }
    }
}
