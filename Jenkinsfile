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
                // Läuft auf dem gemeinsamen Linux-Android-Pool
                // --------------------------------------------------------
                
                stage('Android Remote') {
                    agent { label 'android' }
                    
                    environment {
                        ANDROID_HOME = '/opt/android-sdk'
                        ANDROID_SDK_ROOT = '/opt/android-sdk'
                        ANDROID_NDK_HOME = '/opt/android-sdk/ndk/28.2.13676358'
                        JAVA_HOME = '/usr/lib/jvm/java-17-openjdk-amd64'
                        PATH = "/opt/flutter/bin:/opt/android-sdk/platform-tools:/opt/android-sdk/build-tools/35.0.0:/usr/lib/jvm/java-17-openjdk-amd64/bin:${env.PATH}"
                        VERSION_CODE = "${env.BUILD_NUMBER}"
                    }
                    
                    steps {
                        unstash 'source'
                        
                        echo "=== Setting Version to ${APP_VERSION} ==="
                        sh '''
                            set -eu
                            sed -Ei 's/"version": "[0-9]+\\.[0-9]+\\.[0-9]+"/"version": "'"$APP_VERSION"'"/' remote/src-tauri/tauri.conf.json
                            sed -Ei '0,/version = "[0-9]+\\.[0-9]+\\.[0-9]+"/s//version = "'"$APP_VERSION"'"/' remote/src-tauri/Cargo.toml
                        '''
                        
                        echo '=== Android SDK prüfen ==='
                        sh '''
                            set -eu
                            test -x "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
                            test -d "$ANDROID_NDK_HOME"
                            java -version
                        '''
                        
                        echo '=== Installing Rust Android Targets ==='
                        sh '''
                            set -eu
                            rustup target add aarch64-linux-android
                            rustup target add armv7-linux-androideabi
                            rustup target add x86_64-linux-android
                            rustup target add i686-linux-android
                        '''
                        
                        echo '=== Installing Remote Dependencies ==='
                        dir('remote') {
                            sh 'npm install'
                        }
                        
                        echo '=== Building Android APK ==='
                        dir('remote') {
                            withCredentials([
                                file(credentialsId: 'release.keystore', variable: 'KEYSTORE_FILE'),
                                string(credentialsId: 'keystore-password', variable: 'KEYSTORE_PASSWORD'),
                                string(credentialsId: 'key-alias', variable: 'KEY_ALIAS'),
                                string(credentialsId: 'key-password', variable: 'KEY_PASSWORD')
                            ]) {
                                sh '''
                                    set -eu
                                    install -m 600 "$KEYSTORE_FILE" src-tauri/gen/android/app/release.keystore
                                    printf '%s\n' \
                                        'storeFile=release.keystore' \
                                        "storePassword=$KEYSTORE_PASSWORD" \
                                        "keyAlias=$KEY_ALIAS" \
                                        "keyPassword=$KEY_PASSWORD" \
                                        > src-tauri/gen/android/keystore.properties
                                    npx tauri android build --apk --ci
                                '''
                            }
                        }
                        
                        echo '=== Collecting Android Artifacts ==='
                        sh '''
                            set -eu
                            mkdir -p dist/android
                            find remote/src-tauri/gen/android -type f -name '*.apk' -print -exec cp {} dist/android/ \\;
                            test -n "$(find dist/android -maxdepth 1 -type f -name '*.apk' -print -quit)"
                        '''
                        
                        archiveArtifacts artifacts: 'dist/android/*.apk', fingerprint: true
                        stash includes: 'dist/android/*.apk', name: 'android-apk'
                    }
                    
                    post {
                        always {
                            // Keystore-Dateien entfernen
                            sh 'rm -f remote/src-tauri/gen/android/app/release.keystore remote/src-tauri/gen/android/keystore.properties'
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
        
        // ============================================================
        // STAGE 4: GitHub Release erstellen
        // ============================================================
        
        stage('GitHub Release') {
            agent { label 'windows' }
            
            steps {
                script {
                    // Alle Artifacts sammeln
                    try { unstash 'windows-app' } catch (e) { echo "Windows App: ${e}" }
                    try { unstash 'windows-server' } catch (e) { echo "Windows Server: ${e}" }
                    try { unstash 'linux-packages' } catch (e) { echo "Linux: ${e}" }
                    try { unstash 'android-apk' } catch (e) { echo "Android: ${e}" }
                }
                
                echo "=== Creating GitHub Release v${APP_VERSION} ==="
                
                withCredentials([string(credentialsId: 'freezweb', variable: 'GITHUB_TOKEN')]) {
                    bat """
                        @echo off
                        setlocal enabledelayedexpansion
                        
                        set "VERSION=${APP_VERSION}"
                        set "REPO=freezweb/de.anesda.audiomultiverse"
                        set "TAG=v%VERSION%"
                        
                        echo Creating GitHub Release %TAG%...
                        
                        REM Release-Notizen generieren
                        echo ## AudioMultiverse v%VERSION% > release_notes.md
                        echo. >> release_notes.md
                        echo ### Downloads >> release_notes.md
                        echo. >> release_notes.md
                        echo **Windows:** >> release_notes.md
                        echo - AudioMultiverse_*.msi - Desktop App Installer >> release_notes.md
                        echo - AudioMultiverse_*-setup.exe - Desktop App NSIS Installer >> release_notes.md
                        echo - audiomultiverse-server.exe - Server Binary >> release_notes.md
                        echo. >> release_notes.md
                        echo **Linux:** >> release_notes.md
                        echo - audio-multiverse_*.deb - Desktop App >> release_notes.md
                        echo - audiomultiverse-server_*.deb - Server >> release_notes.md
                        echo. >> release_notes.md
                        echo **Android:** >> release_notes.md
                        echo - app-universal-release.apk - Remote Control App >> release_notes.md
                        echo. >> release_notes.md
                        echo --- >> release_notes.md
                        echo Build: #%BUILD_NUMBER% >> release_notes.md
                        
                        REM GitHub Release erstellen via API
                        curl -s -X POST ^
                            -H "Authorization: token %GITHUB_TOKEN%" ^
                            -H "Accept: application/vnd.github.v3+json" ^
                            "https://api.github.com/repos/%REPO%/releases" ^
                            -d "{\\"tag_name\\": \\"%TAG%\\", \\"name\\": \\"AudioMultiverse v%VERSION%\\", \\"body\\": \\"Release v%VERSION% - Build #%BUILD_NUMBER%\\", \\"draft\\": false, \\"prerelease\\": false}" ^
                            > release_response.json
                        
                        REM Release ID extrahieren
                        for /f "tokens=2 delims=:," %%a in ('findstr /c:"\"id\":" release_response.json ^| findstr /n "^" ^| findstr "^1:"') do (
                            set "RELEASE_ID=%%a"
                        )
                        set "RELEASE_ID=!RELEASE_ID: =!"
                        echo Release ID: !RELEASE_ID!
                        
                        if "!RELEASE_ID!"=="" (
                            echo ERROR: Could not create release
                            type release_response.json
                            exit /b 1
                        )
                        
                        REM Artefakte hochladen
                        echo Uploading artifacts...
                        
                        for %%f in (dist\\windows\\app\\*.msi dist\\windows\\app\\*.exe) do (
                            echo Uploading %%~nxf...
                            curl -s -X POST ^
                                -H "Authorization: token %GITHUB_TOKEN%" ^
                                -H "Content-Type: application/octet-stream" ^
                                "https://uploads.github.com/repos/%REPO%/releases/!RELEASE_ID!/assets?name=%%~nxf" ^
                                --data-binary "@%%f"
                        )
                        
                        for %%f in (dist\\windows\\server\\*.exe) do (
                            echo Uploading %%~nxf...
                            curl -s -X POST ^
                                -H "Authorization: token %GITHUB_TOKEN%" ^
                                -H "Content-Type: application/octet-stream" ^
                                "https://uploads.github.com/repos/%REPO%/releases/!RELEASE_ID!/assets?name=%%~nxf" ^
                                --data-binary "@%%f"
                        )
                        
                        for %%f in (dist\\linux\\*.deb) do (
                            echo Uploading %%~nxf...
                            curl -s -X POST ^
                                -H "Authorization: token %GITHUB_TOKEN%" ^
                                -H "Content-Type: application/octet-stream" ^
                                "https://uploads.github.com/repos/%REPO%/releases/!RELEASE_ID!/assets?name=%%~nxf" ^
                                --data-binary "@%%f"
                        )
                        
                        for %%f in (dist\\android\\*.apk) do (
                            echo Uploading %%~nxf...
                            curl -s -X POST ^
                                -H "Authorization: token %GITHUB_TOKEN%" ^
                                -H "Content-Type: application/vnd.android.package-archive" ^
                                "https://uploads.github.com/repos/%REPO%/releases/!RELEASE_ID!/assets?name=%%~nxf" ^
                                --data-binary "@%%f"
                        )
                        
                        echo.
                        echo GitHub Release v%VERSION% created successfully!
                        echo https://github.com/%REPO%/releases/tag/%TAG%
                    """
                }
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
