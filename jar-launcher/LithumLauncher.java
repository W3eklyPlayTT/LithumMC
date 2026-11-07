package dev.lithum;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;

/** Starts the bundled Windows build of the Lithum Rust server. */
public final class LithumLauncher {
    private LithumLauncher() {}

    public static void main(String[] args) {
        try {
            Path extractionDirectory = Files.createTempDirectory("lithum-server-");
            extractionDirectory.toFile().deleteOnExit();
            Path server = extractionDirectory.resolve("lithum-server.exe");

            try (InputStream bundledServer = LithumLauncher.class.getResourceAsStream(
                    "/native/lithum-server.exe")) {
                if (bundledServer == null) {
                    throw new IOException("The bundled Lithum server executable is missing.");
                }
                Files.copy(bundledServer, server);
            }
            server.toFile().deleteOnExit();

            ArrayList<String> command = new ArrayList<>();
            command.add(server.toAbsolutePath().toString());
            command.addAll(Arrays.asList(args));

            Process process = new ProcessBuilder(command)
                    .directory(Path.of(System.getProperty("user.dir")).toFile())
                    .inheritIO()
                    .start();
            int exitCode = process.waitFor();
            if (exitCode != 0) {
                System.exit(exitCode);
            }
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            System.err.println("Lithum was interrupted while stopping.");
            System.exit(130);
        } catch (IOException error) {
            System.err.println("Could not start Lithum: " + error.getMessage());
            System.exit(1);
        }
    }
}
