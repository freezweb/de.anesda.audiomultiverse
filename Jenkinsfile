// AudioMultiverse CI/CD Pipeline
// Jenkins läuft auf Windows Server
// Benötigte Agents: windows, linux

pipeline {
    agent none
    
    environment {
        APP_NAME = 'AudioMultiverse'
        // Version: Major.Minor aus Datei, Patch = BUILD_NUMBER
        VERSION_MAJOR = '0'
        VERSION_MINOR = '1'
        APP_VERSION = "${VERSION_MAJOR}.${VERSION_MINOR}.${BUILD_NUMBER}"
        RUST_VERSION = '1.75.0'
        NODE_VERSION = '20'
    }
    
    options {
        buildDiscarder(logRotator(numToKeepStr: '3'))
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
                        
                        echo "=== Setting Version to ${APP_VERSION} ==="
                        bat """
                            @echo off
                            setlocal enabledelayedexpansion
                            
                            REM Version in package.json aktualisieren
                            cd app
                            call npm version %APP_VERSION% --no-git-tag-version --allow-same-version
                            cd ..
                            
                            REM Version in tauri.conf.json aktualisieren
                            powershell -Command "(Get-Content app\\src-tauri\\tauri.conf.json) -replace '\"version\": \"[0-9]+\\.[0-9]+\\.[0-9]+\"', '\"version\": \"%APP_VERSION%\"' | Set-Content app\\src-tauri\\tauri.conf.json"
                            
                            REM Version in Cargo.toml aktualisieren (app)
                            powershell -Command "(Get-Content app\\src-tauri\\Cargo.toml) -replace 'version = \"[0-9]+\\.[0-9]+\\.[0-9]+\"', 'version = \"%APP_VERSION%\"' | Set-Content app\\src-tauri\\Cargo.toml"
                            
                            REM Version in Cargo.toml aktualisieren (server)
                            powershell -Command "(Get-Content server\\Cargo.toml) -replace 'version = \"[0-9]+\\.[0-9]+\\.[0-9]+\"', 'version = \"%APP_VERSION%\"' | Set-Content server\\Cargo.toml"
                            
                            echo Version gesetzt: %APP_VERSION%
                        """
                        
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
                            catchError(buildResult: 'SUCCESS', stageResult: 'SUCCESS') {
                                cleanWs()
                            }
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
                        
                        echo "=== Setting Version to ${APP_VERSION} ==="
                        bat """
                            @echo off
                            powershell -Command "(Get-Content server\\Cargo.toml) -replace 'version = \"[0-9]+\\.[0-9]+\\.[0-9]+\"', 'version = \"%APP_VERSION%\"' | Set-Content server\\Cargo.toml"
                        """
                        
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
                            catchError(buildResult: 'SUCCESS', stageResult: 'SUCCESS') {
                                cleanWs()
                            }
                        }
                    }
                }
                
                // --------------------------------------------------------
                // Android Remote App (APK)
                // Läuft auf Windows mit Android SDK
                // --------------------------------------------------------
                
                stage('Android Remote') {
                    agent { label 'windows' }
                    
                    environment {
                        ANDROID_HOME = 'C:\\Program Files (x86)\\Android\\android-sdk'
                        JAVA_HOME = 'C:\\Program Files\\Eclipse Adoptium\\jdk-17.0.16.8-hotspot'
                        PATH = "${env.ANDROID_HOME}\\platform-tools;${env.ANDROID_HOME}\\build-tools\\35.0.0;${env.JAVA_HOME}\\bin;${env.PATH}"
                        VERSION_CODE = "${env.BUILD_NUMBER}"
                    }
                    
                    steps {
                        unstash 'source'
                        
                        echo "=== Setting Version to ${APP_VERSION} ==="
                        bat """
                            @echo off
                            powershell -Command "(Get-Content remote\\src-tauri\\tauri.conf.json) -replace '\"version\": \"[0-9]+\\.[0-9]+\\.[0-9]+\"', '\"version\": \"%APP_VERSION%\"' | Set-Content remote\\src-tauri\\tauri.conf.json"
                            powershell -Command "(Get-Content remote\\src-tauri\\Cargo.toml) -replace 'version = \"[0-9]+\\.[0-9]+\\.[0-9]+\"', 'version = \"%APP_VERSION%\"' | Set-Content remote\\src-tauri\\Cargo.toml"
                        """
                        
                        echo '=== Accepting Android SDK Licenses ==='
                        bat '''
                            @echo off
                            setlocal enabledelayedexpansion
                            
                            REM Erstelle licenses Ordner falls nicht vorhanden
                            if not exist "%ANDROID_HOME%\\licenses" mkdir "%ANDROID_HOME%\\licenses"
                            
                            REM Akzeptiere Lizenzen
                            if exist "%ANDROID_HOME%\\cmdline-tools\\latest\\bin\\sdkmanager.bat" (
                                (echo y & echo y & echo y & echo y & echo y & echo y) | "%ANDROID_HOME%\\cmdline-tools\\latest\\bin\\sdkmanager.bat" --licenses
                            ) else (
                                echo 24333f8a63b6825ea9c5514f83c2829b004d1fee > "%ANDROID_HOME%\\licenses\\android-sdk-license"
                                echo 8933bad161af4178b1185d1a37fbf41ea5269c55 > "%ANDROID_HOME%\\licenses\\android-ndk-license"
                            )
                        '''
                        
                        echo '=== Installing Rust Android Targets ==='
                        bat '''
                            @echo off
                            set "PATH=%USERPROFILE%\\.cargo\\bin;%PATH%"
                            rustup target add aarch64-linux-android
                            rustup target add armv7-linux-androideabi
                            rustup target add x86_64-linux-android
                            rustup target add i686-linux-android
                        '''
                        
                        echo '=== Installing Remote Dependencies ==='
                        dir('remote') {
                            bat '''
                                @echo off
                                call npm install
                            '''
                        }
                        
                        echo '=== Building Android APK ==='
                        dir('remote') {
                            withCredentials([
                                file(credentialsId: 'release.keystore', variable: 'KEYSTORE_FILE'),
                                string(credentialsId: 'keystore-password', variable: 'KEYSTORE_PASSWORD'),
                                string(credentialsId: 'key-alias', variable: 'KEY_ALIAS'),
                                string(credentialsId: 'key-password', variable: 'KEY_PASSWORD')
                            ]) {
                                bat '''
                                    @echo off
                                    setlocal enabledelayedexpansion
                                    set "PATH=%USERPROFILE%\\.cargo\\bin;%PATH%"
                                    
                                    REM Kopiere Keystore in Android-Projekt
                                    copy "%KEYSTORE_FILE%" "src-tauri\\gen\\android\\app\\release.keystore"
                                    
                                    REM Erstelle keystore.properties ohne trailing spaces
                                    set "PROPS_FILE=src-tauri\\gen\\android\\keystore.properties"
                                    (
                                        echo storeFile=release.keystore
                                        echo storePassword=!KEYSTORE_PASSWORD!
                                        echo keyAlias=!KEY_ALIAS!
                                        echo keyPassword=!KEY_PASSWORD!
                                    ) > "!PROPS_FILE!"
                                    
                                    REM Baue Android APK
                                    call npx tauri android build --apk true --ci
                                '''
                            }
                        }
                        
                        echo '=== Collecting Android Artifacts ==='
                        bat '''
                            @echo off
                            mkdir dist\\android 2>nul
                            
                            REM Suche nach APKs in verschiedenen möglichen Pfaden
                            for /r remote\\src-tauri\\gen\\android %%f in (*.apk) do (
                                echo Gefunden: %%f
                                copy "%%f" dist\\android\\
                            )
                        '''
                        
                        archiveArtifacts artifacts: 'dist/android/*.apk', fingerprint: true
                        stash includes: 'dist/android/*.apk', name: 'android-apk'
                    }
                    
                    post {
                        always {
                            // Keystore-Dateien entfernen
                            bat '''
                                @echo off
                                del /q remote\\src-tauri\\gen\\android\\app\\release.keystore 2>nul
                                del /q remote\\src-tauri\\gen\\android\\keystore.properties 2>nul
                            '''
                            // Workspace-Cleanup - Fehler ignorieren (Dateien könnten gesperrt sein)
                            catchError(buildResult: 'SUCCESS', stageResult: 'SUCCESS') {
                                cleanWs()
                            }
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
                        // Manueller Git-Clone da Jenkins git.exe statt git verwendet
                        sh '''
                            cd "$WORKSPACE"
                            if [ -d ".git" ]; then
                                git fetch --all
                                git reset --hard origin/main
                            else
                                git clone https://github.com/freezweb/de.anesda.audiomultiverse.git .
                            fi
                        '''
                        
                        echo "=== Setting Version to ${APP_VERSION} ==="
                        sh """
                            cd "\$WORKSPACE"
                            
                            # Version in package.json aktualisieren
                            cd app
                            npm version ${APP_VERSION} --no-git-tag-version --allow-same-version || true
                            cd ..
                            
                            # Version in tauri.conf.json aktualisieren
                            sed -i 's/"version": "[0-9]*\\.[0-9]*\\.[0-9]*"/"version": "${APP_VERSION}"/' app/src-tauri/tauri.conf.json
                            
                            # Version in Cargo.toml aktualisieren
                            sed -i 's/^version = "[0-9]*\\.[0-9]*\\.[0-9]*"/version = "${APP_VERSION}"/' app/src-tauri/Cargo.toml
                            sed -i 's/^version = "[0-9]*\\.[0-9]*\\.[0-9]*"/version = "${APP_VERSION}"/' server/Cargo.toml
                            
                            echo "Version gesetzt: ${APP_VERSION}"
                        """
                        
                        echo '=== Installing Linux Dependencies ==='
                        sh '''
                            # Rust
                            command -v rustc || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                            . "$HOME/.cargo/env"
                            
                            # System Dependencies für Tauri (Agent läuft als root)
                            apt-get update
                            apt-get install -y \
                                build-essential \
                                libwebkit2gtk-4.0-dev \
                                libappindicator3-dev \
                                librsvg2-dev \
                                patchelf \
                                libssl-dev \
                                libasound2-dev \
                                libjack-jackd2-dev \
                                libsoup2.4-dev \
                                libgtk-3-dev \
                                libjavascriptcoregtk-4.0-dev
                        '''
                        
                        echo '=== Building Server ==='
                        dir('server') {
                            sh '''
                                . "$HOME/.cargo/env"
                                cargo build --release
                            '''
                        }
                        
                        echo '=== Building App ==='
                        dir('app') {
                            sh '''
                                npm install
                                . "$HOME/.cargo/env"
                                npx tauri build --bundles deb
                            '''
                        }
                        
                        echo '=== Creating Server .deb Package ==='
                        sh '''
                            . "$HOME/.cargo/env"
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
                            catchError(buildResult: 'SUCCESS', stageResult: 'SUCCESS') {
                                cleanWs()
                            }
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
                    try { unstash 'android-apk' } catch (e) { echo "Android: ${e}" }
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
