// AudioMultiverse CI/CD Pipeline
// Jenkins läuft auf Windows Server
// Benötigte Agents: windows, linux, android

pipeline {
    agent none
    
    environment {
        APP_NAME = 'AudioMultiverse'
        APP_VERSION = '0.1.0'
        RUST_VERSION = '1.75.0'
        NODE_VERSION = '20'
        PNPM_VERSION = '8'
    }
    
    options {
        buildDiscarder(logRotator(numToKeepStr: '10'))
        timestamps()
        timeout(time: 2, unit: 'HOURS')
    }
    
    stages {
        
        // ============================================================
        // STAGE 1: Vorbereitung & Tests
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
                // Windows Remote Client (Inno Setup)
                // --------------------------------------------------------
                
                stage('Windows Remote') {
                    agent { label 'windows' }
                    
                    environment {
                        TAURI_PRIVATE_KEY = credentials('tauri-private-key')
                    }
                    
                    steps {
                        unstash 'source'
                        
                        echo '=== Installing Dependencies ==='
                        bat '''
                            @echo off
                            
                            REM Node.js & pnpm prüfen
                            where node >nul 2>&1 || (
                                echo Node.js nicht gefunden!
                                exit /b 1
                            )
                            
                            REM pnpm installieren falls nicht vorhanden
                            where pnpm >nul 2>&1 || npm install -g pnpm@%PNPM_VERSION%
                            
                            REM Rust prüfen
                            where rustc >nul 2>&1 || (
                                echo Rust nicht gefunden!
                                exit /b 1
                            )
                            
                            REM Dependencies installieren
                            pnpm install --frozen-lockfile
                        '''
                        
                        echo '=== Building Remote Client ==='
                        dir('remote') {
                            bat '''
                                @echo off
                                pnpm tauri build
                            '''
                        }
                        
                        echo '=== Creating Inno Setup Installer ==='
                        bat '''
                            @echo off
                            
                            REM Inno Setup Compiler Pfad
                            set ISCC="C:\\Program Files (x86)\\Inno Setup 6\\ISCC.exe"
                            
                            REM Installer bauen
                            %ISCC% /O"dist\\windows" /F"AudioMultiverse-Remote-Setup-%APP_VERSION%" "deploy\\innosetup\\remote.iss"
                        '''
                        
                        archiveArtifacts artifacts: 'dist/windows/*.exe', fingerprint: true
                        stash includes: 'dist/windows/*.exe', name: 'windows-installer'
                    }
                    
                    post {
                        always {
                            cleanWs()
                        }
                    }
                }
                
                // --------------------------------------------------------
                // Linux Server & App (.deb Pakete)
                // --------------------------------------------------------
                
                stage('Linux Packages') {
                    agent { label 'linux' }
                    
                    environment {
                        TAURI_PRIVATE_KEY = credentials('tauri-private-key')
                    }
                    
                    steps {
                        unstash 'source'
                        
                        echo '=== Installing Dependencies ==='
                        sh '''
                            # Node.js & pnpm
                            export PATH="$HOME/.local/share/pnpm:$PATH"
                            command -v pnpm || npm install -g pnpm@${PNPM_VERSION}
                            
                            # Rust
                            command -v rustc || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                            source "$HOME/.cargo/env"
                            
                            # System Dependencies für Tauri
                            sudo apt-get update
                            sudo apt-get install -y \
                                libwebkit2gtk-4.1-dev \
                                libappindicator3-dev \
                                librsvg2-dev \
                                patchelf \
                                libssl-dev \
                                libasound2-dev \
                                libjack-jackd2-dev
                            
                            # pnpm Dependencies
                            pnpm install --frozen-lockfile
                        '''
                        
                        echo '=== Building Server ==='
                        dir('server') {
                            sh '''
                                source "$HOME/.cargo/env"
                                cargo build --release
                            '''
                        }
                        
                        echo '=== Building App (Local UI) ==='
                        dir('app') {
                            sh '''
                                source "$HOME/.cargo/env"
                                pnpm tauri build --bundles deb
                            '''
                        }
                        
                        echo '=== Building Remote Client ==='
                        dir('remote') {
                            sh '''
                                source "$HOME/.cargo/env"
                                pnpm tauri build --bundles deb
                            '''
                        }
                        
                        echo '=== Creating Server .deb Package ==='
                        sh '''
                            # Server .deb bauen mit cargo-deb
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
                            find remote/src-tauri/target/release/bundle/deb -name "*.deb" -exec cp {} dist/linux/ \\;
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
                
                // --------------------------------------------------------
                // Linux ARM (Raspberry Pi)
                // --------------------------------------------------------
                
                stage('Raspberry Pi') {
                    agent { label 'linux-arm64' }
                    
                    steps {
                        unstash 'source'
                        
                        echo '=== Building for Raspberry Pi ==='
                        sh '''
                            source "$HOME/.cargo/env"
                            
                            # Cross-Compile für ARM64
                            rustup target add aarch64-unknown-linux-gnu
                            
                            cd server
                            cargo build --release --target aarch64-unknown-linux-gnu
                            
                            # .deb für ARM erstellen
                            cargo deb --target aarch64-unknown-linux-gnu --output ../dist/raspberry-pi/
                        '''
                        
                        archiveArtifacts artifacts: 'dist/raspberry-pi/*.deb', fingerprint: true
                        stash includes: 'dist/raspberry-pi/*.deb', name: 'rpi-packages'
                    }
                    
                    post {
                        always {
                            cleanWs()
                        }
                    }
                }
                
                // --------------------------------------------------------
                // Android APK
                // --------------------------------------------------------
                
                stage('Android APK') {
                    agent { label 'android' }
                    
                    environment {
                        ANDROID_HOME = '/opt/android-sdk'
                        ANDROID_NDK_HOME = '/opt/android-sdk/ndk/25.2.9519653'
                        JAVA_HOME = '/usr/lib/jvm/java-17-openjdk-amd64'
                        TAURI_SIGNING_KEY = credentials('android-keystore')
                        TAURI_KEY_PASSWORD = credentials('android-keystore-password')
                    }
                    
                    steps {
                        unstash 'source'
                        
                        echo '=== Setting up Android Environment ==='
                        sh '''
                            source "$HOME/.cargo/env"
                            
                            # Rust Android Targets
                            rustup target add aarch64-linux-android
                            rustup target add armv7-linux-androideabi
                            rustup target add x86_64-linux-android
                            
                            # Node Dependencies
                            pnpm install --frozen-lockfile
                        '''
                        
                        echo '=== Building Android Remote ==='
                        dir('remote') {
                            sh '''
                                source "$HOME/.cargo/env"
                                
                                # Tauri Android initialisieren (falls noch nicht)
                                pnpm tauri android init || true
                                
                                # APK bauen
                                pnpm tauri android build --apk
                            '''
                        }
                        
                        echo '=== Signing APK ==='
                        sh '''
                            mkdir -p dist/android
                            
                            # Unsigned APK finden
                            APK_PATH=$(find remote/src-tauri/gen/android -name "*-unsigned.apk" | head -1)
                            
                            # Signieren
                            ${ANDROID_HOME}/build-tools/34.0.0/apksigner sign \\
                                --ks ${TAURI_SIGNING_KEY} \\
                                --ks-pass pass:${TAURI_KEY_PASSWORD} \\
                                --out dist/android/AudioMultiverse-Remote-${APP_VERSION}.apk \\
                                "$APK_PATH"
                            
                            # Verify
                            ${ANDROID_HOME}/build-tools/34.0.0/apksigner verify dist/android/*.apk
                        '''
                        
                        archiveArtifacts artifacts: 'dist/android/*.apk', fingerprint: true
                        stash includes: 'dist/android/*.apk', name: 'android-apk'
                    }
                    
                    post {
                        always {
                            cleanWs()
                        }
                    }
                }
                
                // --------------------------------------------------------
                // macOS (Optional - erfordert macOS Agent)
                // --------------------------------------------------------
                
                stage('macOS') {
                    agent { label 'macos' }
                    
                    when {
                        expression { 
                            // Nur bauen wenn macOS Agent verfügbar
                            return env.BUILD_MACOS == 'true' 
                        }
                    }
                    
                    environment {
                        TAURI_PRIVATE_KEY = credentials('tauri-private-key')
                        APPLE_CERTIFICATE = credentials('apple-developer-cert')
                        APPLE_CERTIFICATE_PASSWORD = credentials('apple-cert-password')
                        APPLE_ID = credentials('apple-id')
                        APPLE_TEAM_ID = credentials('apple-team-id')
                    }
                    
                    steps {
                        unstash 'source'
                        
                        sh '''
                            # pnpm & Dependencies
                            npm install -g pnpm@${PNPM_VERSION}
                            pnpm install --frozen-lockfile
                        '''
                        
                        dir('remote') {
                            sh '''
                                pnpm tauri build --bundles dmg
                            '''
                        }
                        
                        sh '''
                            mkdir -p dist/macos
                            find remote/src-tauri/target/release/bundle/dmg -name "*.dmg" -exec cp {} dist/macos/ \\;
                        '''
                        
                        archiveArtifacts artifacts: 'dist/macos/*.dmg', fingerprint: true
                        stash includes: 'dist/macos/*.dmg', name: 'macos-dmg'
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
        // STAGE 3: Artifacts sammeln & Release
        // ============================================================
        
        stage('Collect Artifacts') {
            agent { label 'windows' }
            
            steps {
                script {
                    // Alle Artifacts sammeln
                    try { unstash 'windows-installer' } catch (e) { echo "Windows: ${e}" }
                    try { unstash 'linux-packages' } catch (e) { echo "Linux: ${e}" }
                    try { unstash 'rpi-packages' } catch (e) { echo "RPi: ${e}" }
                    try { unstash 'android-apk' } catch (e) { echo "Android: ${e}" }
                    try { unstash 'macos-dmg' } catch (e) { echo "macOS: ${e}" }
                }
                
                echo '=== All Artifacts ==='
                bat 'dir /s /b dist'
                
                // Release-Ordner erstellen
                bat '''
                    @echo off
                    mkdir release\\%APP_VERSION% 2>nul
                    xcopy /E /Y dist\\* release\\%APP_VERSION%\\
                '''
                
                archiveArtifacts artifacts: 'release/**/*', fingerprint: true
            }
        }
        
        // ============================================================
        // STAGE 4: Deployment (Optional)
        // ============================================================
        
        stage('Deploy') {
            agent { label 'windows' }
            
            when {
                branch 'main'
                expression { return params.DEPLOY == true }
            }
            
            steps {
                echo '=== Deploying to Release Server ==='
                
                // Upload zu GitHub Releases oder eigenem Server
                bat '''
                    @echo off
                    echo Deployment would happen here...
                    REM scp -r release\\%APP_VERSION% user@server:/releases/
                '''
            }
        }
    }
    
    // ============================================================
    // POST-ACTIONS
    // ============================================================
    
    post {
        success {
            echo 'Build erfolgreich!'
            
            // Optional: Notification
            // emailext subject: "Build Success: ${APP_NAME} ${APP_VERSION}",
            //     body: "Alle Artefakte wurden erfolgreich erstellt.",
            //     to: 'team@example.com'
        }
        
        failure {
            echo 'Build fehlgeschlagen!'
            
            // emailext subject: "Build FAILED: ${APP_NAME} ${APP_VERSION}",
            //     body: "Build ist fehlgeschlagen. Bitte prüfen.",
            //     to: 'team@example.com'
        }
        
        always {
            echo "Pipeline beendet: ${currentBuild.result}"
        }
    }
}

// ============================================================
// Parameter für manuellen Trigger
// ============================================================

properties([
    parameters([
        booleanParam(
            name: 'DEPLOY',
            defaultValue: false,
            description: 'Deploy nach erfolgreichem Build?'
        ),
        booleanParam(
            name: 'BUILD_MACOS',
            defaultValue: false,
            description: 'macOS Build erstellen? (benötigt macOS Agent)'
        ),
        string(
            name: 'VERSION_OVERRIDE',
            defaultValue: '',
            description: 'Version überschreiben (leer = aus package.json)'
        )
    ])
])
