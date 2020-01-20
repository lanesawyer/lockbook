import 'package:client/either.dart';
import 'package:client/encryption_helper.dart';
import 'package:client/errors.dart';
import 'package:client/network_helper.dart';
import 'package:client/user_info.dart';
import 'package:client/user_repository.dart';

class AccountHelper {
  final EncryptionHelper encryptionHelper;
  final UserRepository userRepo;
  final NetworkHelper networkHelper;

  const AccountHelper(this.encryptionHelper, this.userRepo, this.networkHelper);

  Future<Either<UIError, UserInfo>> newAccount(String username) async {
    final keyPair = encryptionHelper.generateKeyPair();
    final userInfo = UserInfo.fromAsymmetricKey(username, keyPair);

    final saveUserInfo = await userRepo.saveUserInfo(userInfo);

    final upload = await saveUserInfo
        .flatMapFut<Empty>((_) => networkHelper.newAccount());

    return upload.map((_) => userInfo);
  }
}
