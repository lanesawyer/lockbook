package app.lockbook.util

import com.beust.klaxon.Json
import java.util.*

data class FileMetadata(
    val id: String = "",
    @Json(name = "file_type")
    val fileType: FileType = FileType.Document,
    val parent: String = "",
    val name: String = "",
    val owner: String = "",
    val signature: SignedValue = SignedValue(),
    @Json(name = "metadata_version")
    val metadataVersion: Long = 0,
    @Json(name = "content_version")
    val contentVersion: Long = 0,
    val deleted: Boolean = false,
    @Json(name = "user_access_keys")
    val userAccessKeys: LinkedHashMap<String, UserAccessInfo> = linkedMapOf(),
    @Json(name = "folder_access_keys")
    val folderAccessKeys: FolderAccessInfo = FolderAccessInfo()
)

data class FileUsage(
    @Json(name = "file_id")
    val fileId: String,
    @Json(name = "byte_secs")
    val byteSections: Int,
    val secs: Int,
)

data class SignedValue(
    val content: String = "",
    val signature: String = ""
)

data class FolderAccessInfo(
    @Json(name = "folder_id")
    val folderId: String = "",
    @Json(name = "access_key")
    val accessKey: EncryptedValueWithNonce = EncryptedValueWithNonce()
)

data class EncryptedValueWithNonce(
    val garbage: String = "",
    val nonce: String = ""
)

enum class FileType {
    Document, Folder
}

data class UserAccessInfo(
    val username: String,
    @Json(name = "public_key")
    val publicKey: RSAPublicKey,
    @Json(name = "access_key")
    val accessKey: EncryptedValue
)

data class EncryptedValue(
    val garbage: String
)

data class DecryptedValue(
    val secret: String
)

data class Account(
    val username: String,
    val keys: RSAPrivateKey,
    @Json(name = "api_url")
    val apiUrl: String,
)

data class RSAPrivateKey(
    val n: List<Int>,
    val e: List<Int>,
    val d: List<Int>,
    val primes: List<String>
)

data class RSAPublicKey(
    val n: List<Int>,
    val e: List<Int>
)

data class WorkCalculated(
    @Json(name = "work_units")
    val workUnits: List<WorkUnit>,
    @Json(name = "most_recent_update_from_server")
    val mostRecentUpdateFromServer: Long
)

data class WorkUnit(
    val tag: String,
    val content: WorkUnitMetadata
)

data class WorkUnitMetadata(val metadata: FileMetadata)

data class Config(val writeable_path: String)

enum class State {
    ReadyToUse,
    Empty,
    MigrationRequired,
    StateRequiresClearing
}

data class EditableFile(
    val name: String,
    val id: String,
)

data class SyncingStatus(
    var isSyncing: Boolean = false,
    var maxProgress: Int = 0
)

data class CreateFileDialogInfo(
    var isDialogOpen: Boolean = false,
    var alertDialogFileName: String = "",
    var fileCreationType: FileType = FileType.Document
)

data class RenameFileDialogInfo(
    var isDialogOpen: Boolean = false,
    var alertDialogFileName: String = ""
)

data class MoveFileInfo(
    val ids: Array<String>,
    val names: Array<String>
)

data class RenameFileInfo(
    val id: String,
    val name: String
)

data class CreateFileInfo(
    val parentId: String,
    val fileType: String
)

data class Drawing(
    val currentView: Page = Page(),
    val events: MutableList<Event> = mutableListOf()
)

data class Event(
    val stroke: Stroke? = null
)

data class Stroke(
    val color: Int,
    val points: MutableList<Float> = mutableListOf()
)

data class Page(
    var transformation: Transformation = Transformation(),
)

data class Transformation(
    var translation: Point = Point(0f, 0f),
    var scale: Float = 1f,
)

data class PressurePoint(
    val x: Float,
    val y: Float,
    val pressure: Float
)

data class Point(
    var x: Float,
    var y: Float
)