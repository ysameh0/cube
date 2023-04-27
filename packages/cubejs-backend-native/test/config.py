# import aiohttp
# import asyncio

from cube.conf import settings

settings.schema_path = "models"
settings.pg_sql_port = 5555
settings.telemetry = False
settings.query_rewrite = lambda query, ctx: print('query=', query, 'ctx=', ctx)

def check_auth(req, authorization):
    print('check_auth req=', req, ' authorization=', authorization)

settings.check_auth = check_auth
