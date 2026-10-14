@!documentation("Bounded typing, relation, and member lookup outcomes.")

class TypingResult {}
class TypingKnown is TypingResult {}
class TypingUnknown is TypingResult {}
class TypingInvalid is TypingResult {}
class TypingUnavailable is TypingResult {}
class TypingCancelled is TypingResult {}
class TypingBudgetExceeded is TypingResult {}
class TypingInternalFailure is TypingResult {}

class TypeRelationResult {}
class RelationSatisfied is TypeRelationResult {}
class RelationRejected is TypeRelationResult {}
class RelationDynamicBoundary is TypeRelationResult {}
class RelationBlocked is TypeRelationResult {}
class RelationCancelled is TypeRelationResult {}
class RelationBudgetExceeded is TypeRelationResult {}
class RelationInternalFailure is TypeRelationResult {}

class MemberLookupResult {}
class MemberFound is MemberLookupResult {}
class MemberMissing is MemberLookupResult {}
class MemberDynamicBoundary is MemberLookupResult {}
class MemberBlocked is MemberLookupResult {}
class MemberCancelled is MemberLookupResult {}
class MemberBudgetExceeded is MemberLookupResult {}
class MemberInternalFailure is MemberLookupResult {}
