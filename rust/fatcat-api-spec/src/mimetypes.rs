/// mime types for requests and responses

pub mod responses {
    use hyper::mime::*;

    // The macro is called per-operation to beat the recursion limit
    /// Create Mime objects for the response content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainer
    lazy_static! {
        pub static ref DELETE_CONTAINER_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainer
    lazy_static! {
        pub static ref DELETE_CONTAINER_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainer
    lazy_static! {
        pub static ref DELETE_CONTAINER_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainer
    lazy_static! {
        pub static ref DELETE_CONTAINER_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainer
    lazy_static! {
        pub static ref GET_CONTAINER_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainer
    lazy_static! {
        pub static ref GET_CONTAINER_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainer
    lazy_static! {
        pub static ref GET_CONTAINER_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainer
    lazy_static! {
        pub static ref GET_CONTAINER_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerHistory
    lazy_static! {
        pub static ref GET_CONTAINER_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerHistory
    lazy_static! {
        pub static ref GET_CONTAINER_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerHistory
    lazy_static! {
        pub static ref GET_CONTAINER_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerHistory
    lazy_static! {
        pub static ref GET_CONTAINER_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupContainer
    lazy_static! {
        pub static ref LOOKUP_CONTAINER_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupContainer
    lazy_static! {
        pub static ref LOOKUP_CONTAINER_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupContainer
    lazy_static! {
        pub static ref LOOKUP_CONTAINER_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupContainer
    lazy_static! {
        pub static ref LOOKUP_CONTAINER_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreator
    lazy_static! {
        pub static ref DELETE_CREATOR_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreator
    lazy_static! {
        pub static ref DELETE_CREATOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreator
    lazy_static! {
        pub static ref DELETE_CREATOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreator
    lazy_static! {
        pub static ref DELETE_CREATOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreator
    lazy_static! {
        pub static ref GET_CREATOR_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreator
    lazy_static! {
        pub static ref GET_CREATOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreator
    lazy_static! {
        pub static ref GET_CREATOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreator
    lazy_static! {
        pub static ref GET_CREATOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorHistory
    lazy_static! {
        pub static ref GET_CREATOR_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorHistory
    lazy_static! {
        pub static ref GET_CREATOR_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorHistory
    lazy_static! {
        pub static ref GET_CREATOR_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorHistory
    lazy_static! {
        pub static ref GET_CREATOR_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorReleases
    lazy_static! {
        pub static ref GET_CREATOR_RELEASES_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorReleases
    lazy_static! {
        pub static ref GET_CREATOR_RELEASES_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorReleases
    lazy_static! {
        pub static ref GET_CREATOR_RELEASES_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorReleases
    lazy_static! {
        pub static ref GET_CREATOR_RELEASES_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupCreator
    lazy_static! {
        pub static ref LOOKUP_CREATOR_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupCreator
    lazy_static! {
        pub static ref LOOKUP_CREATOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupCreator
    lazy_static! {
        pub static ref LOOKUP_CREATOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupCreator
    lazy_static! {
        pub static ref LOOKUP_CREATOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditor
    lazy_static! {
        pub static ref GET_EDITOR_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditor
    lazy_static! {
        pub static ref GET_EDITOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditor
    lazy_static! {
        pub static ref GET_EDITOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditor
    lazy_static! {
        pub static ref GET_EDITOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditorChangelog
    lazy_static! {
        pub static ref GET_EDITOR_CHANGELOG_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditorChangelog
    lazy_static! {
        pub static ref GET_EDITOR_CHANGELOG_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditorChangelog
    lazy_static! {
        pub static ref GET_EDITOR_CHANGELOG_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditorChangelog
    lazy_static! {
        pub static ref GET_EDITOR_CHANGELOG_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetStats
    lazy_static! {
        pub static ref GET_STATS_SUCCESS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetStats
    lazy_static! {
        pub static ref GET_STATS_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_MERGED_SUCCESSFULLY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_EDIT_CONFLICT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateEditgroup
    lazy_static! {
        pub static ref CREATE_EDITGROUP_SUCCESSFULLY_CREATED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateEditgroup
    lazy_static! {
        pub static ref CREATE_EDITGROUP_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateEditgroup
    lazy_static! {
        pub static ref CREATE_EDITGROUP_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetChangelog
    lazy_static! {
        pub static ref GET_CHANGELOG_SUCCESS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetChangelog
    lazy_static! {
        pub static ref GET_CHANGELOG_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetChangelogEntry
    lazy_static! {
        pub static ref GET_CHANGELOG_ENTRY_FOUND_CHANGELOG_ENTRY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetChangelogEntry
    lazy_static! {
        pub static ref GET_CHANGELOG_ENTRY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetChangelogEntry
    lazy_static! {
        pub static ref GET_CHANGELOG_ENTRY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditgroup
    lazy_static! {
        pub static ref GET_EDITGROUP_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditgroup
    lazy_static! {
        pub static ref GET_EDITGROUP_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditgroup
    lazy_static! {
        pub static ref GET_EDITGROUP_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditgroup
    lazy_static! {
        pub static ref GET_EDITGROUP_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFile
    lazy_static! {
        pub static ref DELETE_FILE_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFile
    lazy_static! {
        pub static ref DELETE_FILE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFile
    lazy_static! {
        pub static ref DELETE_FILE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFile
    lazy_static! {
        pub static ref DELETE_FILE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFile
    lazy_static! {
        pub static ref GET_FILE_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFile
    lazy_static! {
        pub static ref GET_FILE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFile
    lazy_static! {
        pub static ref GET_FILE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFile
    lazy_static! {
        pub static ref GET_FILE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileHistory
    lazy_static! {
        pub static ref GET_FILE_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileHistory
    lazy_static! {
        pub static ref GET_FILE_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileHistory
    lazy_static! {
        pub static ref GET_FILE_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileHistory
    lazy_static! {
        pub static ref GET_FILE_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupFile
    lazy_static! {
        pub static ref LOOKUP_FILE_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupFile
    lazy_static! {
        pub static ref LOOKUP_FILE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupFile
    lazy_static! {
        pub static ref LOOKUP_FILE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupFile
    lazy_static! {
        pub static ref LOOKUP_FILE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteRelease
    lazy_static! {
        pub static ref DELETE_RELEASE_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteRelease
    lazy_static! {
        pub static ref DELETE_RELEASE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteRelease
    lazy_static! {
        pub static ref DELETE_RELEASE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteRelease
    lazy_static! {
        pub static ref DELETE_RELEASE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetRelease
    lazy_static! {
        pub static ref GET_RELEASE_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetRelease
    lazy_static! {
        pub static ref GET_RELEASE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetRelease
    lazy_static! {
        pub static ref GET_RELEASE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetRelease
    lazy_static! {
        pub static ref GET_RELEASE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFiles
    lazy_static! {
        pub static ref GET_RELEASE_FILES_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFiles
    lazy_static! {
        pub static ref GET_RELEASE_FILES_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFiles
    lazy_static! {
        pub static ref GET_RELEASE_FILES_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFiles
    lazy_static! {
        pub static ref GET_RELEASE_FILES_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseHistory
    lazy_static! {
        pub static ref GET_RELEASE_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseHistory
    lazy_static! {
        pub static ref GET_RELEASE_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseHistory
    lazy_static! {
        pub static ref GET_RELEASE_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseHistory
    lazy_static! {
        pub static ref GET_RELEASE_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupRelease
    lazy_static! {
        pub static ref LOOKUP_RELEASE_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupRelease
    lazy_static! {
        pub static ref LOOKUP_RELEASE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupRelease
    lazy_static! {
        pub static ref LOOKUP_RELEASE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupRelease
    lazy_static! {
        pub static ref LOOKUP_RELEASE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWork
    lazy_static! {
        pub static ref DELETE_WORK_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWork
    lazy_static! {
        pub static ref DELETE_WORK_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWork
    lazy_static! {
        pub static ref DELETE_WORK_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWork
    lazy_static! {
        pub static ref DELETE_WORK_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWork
    lazy_static! {
        pub static ref GET_WORK_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWork
    lazy_static! {
        pub static ref GET_WORK_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWork
    lazy_static! {
        pub static ref GET_WORK_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWork
    lazy_static! {
        pub static ref GET_WORK_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkHistory
    lazy_static! {
        pub static ref GET_WORK_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkHistory
    lazy_static! {
        pub static ref GET_WORK_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkHistory
    lazy_static! {
        pub static ref GET_WORK_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkHistory
    lazy_static! {
        pub static ref GET_WORK_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkReleases
    lazy_static! {
        pub static ref GET_WORK_RELEASES_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkReleases
    lazy_static! {
        pub static ref GET_WORK_RELEASES_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkReleases
    lazy_static! {
        pub static ref GET_WORK_RELEASES_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkReleases
    lazy_static! {
        pub static ref GET_WORK_RELEASES_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK_GENERIC_ERROR: Mime = mime!(Application / Json);
    }

}

pub mod requests {
    use hyper::mime::*;
    /// Create Mime objects for the request content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateEditgroup
    lazy_static! {
        pub static ref CREATE_EDITGROUP: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK: Mime = mime!(Application / Json);
    }

}
