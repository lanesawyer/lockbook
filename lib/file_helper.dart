import 'dart:io';

import 'package:client/errors.dart';
import 'package:client/either.dart';
import 'package:path_provider/path_provider.dart';

class FileHelper {
  const FileHelper();

  Future<Either<UIError, Directory>> _getFileStoreDir() async {
    final Directory directory =
        await getApplicationDocumentsDirectory().catchError((dynamic e) {
      print("Error getting application directory, prob plugin not supported");
      print(e);
      // the implementation indicates it may return null, so I'll do that too :(
      return null;
    });

    if (directory == null) {
      return Fail(UIError(
          "Unable to access file system",
          "It seems path_provider is not supported on this platform, "
              "please tell us what platform you're using, and we'll investigate: "
              "github.com/lockbook -> issues"));
    }

    Directory filesFolder =
        await Directory(directory.path + "/files/").create();
    return Success(filesFolder);
  }

  Future<Either<UIError, Empty>> writeToFile(
      String location, String content) async {
    return (await _getFileStoreDir()).flatMap((dir) {
      final file = File(dir.path + location);
      print(file);
      try {
        file.writeAsStringSync(content);
      } catch (error) {
        print(error);
        return Fail(UIError("Could not write to file",
            "Error: $error while writing to $location"));
      }
      return Success(Done);
    });
  }

  Future<Either<UIError, String>> readFromFile(String location) async {
    final getLocation = await _getFileStoreDir();

    return getLocation.flatMap((dir) {
      final file = File(dir.path + location);

      try {
        return Success(file.readAsStringSync());
      } catch (error) {
        return Fail(UIError(
            "Could not read file", "Error: $error while writing to $location"));
      }
    });
  }
}
