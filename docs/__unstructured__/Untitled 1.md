```ph
type data Column(
	name: Symbol,
	type: SqlType,
	nullable: Bool
)

type data Table<**Fs>(
	name: String
)

impl<**Fs> Table<**Fs> {
	type Row = 
}

type UsersRow = {
	id: UserId,
	username: String,
	email: String,
	passwordHash: String,
	createdAt: Instant,
}

type Users = 
	Table<UsersRow>::Table(
		name: "users"
	)

Select<
	Users,
	#id,
	#username
>

type SelectResult<
    T: Table,
    *Columns: Symbol
>: Record =
    Pick<T::Row, *Columns>
    
let query =
	Users
		.select(#id, #username, #email)
		.where(...)
		
Query<{
	id: UserId,
	username: String,
	email: String,
}>

type SelectedRow<
	**Fs,
	Row: Record<**Fs>,
	*Fields: Symbol
>: Record =
	Pick<Row, *Fields>
	
let users = db.execute(query)

List<{
    id: UserId,
    username: String,
    email: String,
}>

type Prefix<
	R: Record<>,
	Prefix: String
>: Record =
	Record<
		**R::Entries::MapEntries |name, T| {
			(
				Symbol("${Prefix}${name}"),
				T
			)
		}
	>
		
type UserJson =
	Omit<
		{
			id: UserId,
			username: String,
			email: String,
			passwordHash: String,
		},
		#passwordHash
	>
```