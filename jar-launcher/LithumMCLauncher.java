package dev.lithummc;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.attribute.PosixFilePermission;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Locale;
import java.util.Set;
import java.util.concurrent.TimeUnit;

/** Starts the bundled native LithumMC Rust server for the current platform. */
public final class LithumMCLauncher {
    private LithumMCLauncher() {}

    public static void main(String[] args) {
        try {
            String osName = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
            String architecture = System.getProperty("os.arch", "").toLowerCase(Locale.ROOT);
            String resourceName;
            String executableName;

            if ((osName.contains("linux"))
                    && (architecture.equals("amd64") || architecture.equals("x86_64"))) {
                resourceName = "/native/lithummc-server-linux-x86_64";
                executableName = "lithummc-server-linux-x86_64";
            } else if (osName.contains("windows")
                    && (architecture.equals("amd64") || architecture.equals("x86_64"))) {
                resourceName = "/native/lithummc-server.exe";
                executableName = "lithummc-server.exe";
            } else {
                throw new IOException(
                        "This JAR supports Windows x86_64 and Linux x86_64. Detected "
                                + osName + " " + architecture + ".");
            }

            Path workingDirectory = Path.of(System.getProperty("user.dir"));
            Path extractionDirectory = Files.createTempDirectory(workingDirectory, ".lithummc-server-");
            extractionDirectory.toFile().deleteOnExit();
            Path server = extractionDirectory.resolve(executableName);

            try (InputStream bundledServer = LithumMCLauncher.class.getResourceAsStream(resourceName)) {
                if (bundledServer == null) {
                    throw new IOException("Bundled server for " + osName + " is missing.");
                }
                Files.copy(bundledServer, server);
            }
            server.toFile().deleteOnExit();
            if (osName.contains("linux")) {
                try {
                    Files.setPosixFilePermissions(server, Set.of(
                            PosixFilePermission.OWNER_READ,
                            PosixFilePermission.OWNER_WRITE,
                            PosixFilePermission.OWNER_EXECUTE));
                } catch (UnsupportedOperationException unsupported) {
                    if (!server.toFile().setExecutable(true, true) && !server.toFile().canExecute()) {
                        throw new IOException("Could not mark the bundled LithumMC server executable.");
                    }
                }
            }

            ArrayList<String> command = new ArrayList<>();
            command.add(server.toAbsolutePath().toString());
            command.addAll(Arrays.asList(args));

            Process process = new ProcessBuilder(command)
                    .directory(Path.of(System.getProperty("user.dir")).toFile())
                    .inheritIO()
                    .start();
            Thread shutdownHook = new Thread(() -> stopServer(process), "lithummc-server-shutdown");
            Runtime.getRuntime().addShutdownHook(shutdownHook);
            int exitCode = process.waitFor();
            try {
                Runtime.getRuntime().removeShutdownHook(shutdownHook);
            } catch (IllegalStateException shutdownStarted) {
                // The shutdown hook owns child cleanup when the JVM is already shutting down.
            }
            if (exitCode != 0) {
                System.exit(exitCode);
            }
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            System.err.println("LithumMC was interrupted while stopping.");
            System.exit(130);
        } catch (IOException error) {
            System.err.println("Could not start LithumMC: " + error.getMessage());
            System.exit(1);
        }
    }

    private static void stopServer(Process process) {
        if (!process.isAlive()) {
            return;
        }
        process.destroy();
        try {
            if (!process.waitFor(20, TimeUnit.SECONDS)) {
                process.destroyForcibly();
            }
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            process.destroyForcibly();
        }
    }
}
